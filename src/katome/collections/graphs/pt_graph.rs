//! `petgraph` based `Graph`.

use algorithms::builder::{Build, Init};
use asm::SEQUENCES;
use collections::graphs::Graph;
use compress::{compress_kmer, kmer_to_edge, compress_kmer_with_rev_compl};
use config::InputFileType;
use prelude::{EdgeWeight, Idx, K_SIZE, K1_SIZE};
use slices::{BasicSlice, EdgeSlice, NodeSlice};

use fixedbitset::FixedBitSet;
use metrohash::MetroHash;
use petgraph;
use petgraph::dot::{Config, Dot};

use std::collections::HashSet;
use std::collections::hash_map::{Entry, HashMap};
use std::error::Error;
use std::fs::File;
use std::hash::BuildHasherDefault as BuildHash;
use std::io::prelude::*;
use std::path::Path;


/// Type denoting index of edge.
pub type EdgeIndex = petgraph::graph::EdgeIndex<Idx>;
/// Type denoting index of node.
pub type NodeIndex = petgraph::graph::NodeIndex<Idx>;
/// `Node` type in `PtGraph`.
pub type Node = petgraph::graph::Node<(), Idx>;

/// `petgraph` based `Graph`.
pub type PtGraph = petgraph::Graph<(), (EdgeSlice, EdgeWeight), petgraph::Directed, Idx>;

/// Serialize graph into .dot file.
pub fn write_to_dot(graph: &PtGraph, path_: &str) {
    let path = Path::new(path_);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why.description()),
        Ok(file) => file,
    };
    match write!(file, "{:?}", Dot::with_config(graph, &[Config::NodeIndexLabel])) {
        Err(why) => panic!("couldn't write to {}: {}", display, why.description()),
        Ok(_) => info!("successfully wrote to {}", display),
    }
}

impl Graph for PtGraph {
    type NodeIdentifier = NodeIndex;
    type AmbiguousNodes = HashSet<NodeIndex>;
    fn get_ambiguous_nodes(&self) -> Self::AmbiguousNodes {
        self.node_indices()
            .filter(|n| {
                let in_degree = self.in_degree(*n);
                let out_degree = self.out_degree(*n);
                (in_degree > 1 || out_degree > 1) || (in_degree == 0 && out_degree >= 1)
            })
            .collect::<Self::AmbiguousNodes>()
    }

    fn out_degree(&self, node: Self::NodeIdentifier) -> usize {
        self.neighbors_directed(node, petgraph::EdgeDirection::Outgoing)
            .count()
    }

    fn in_degree(&self, node: Self::NodeIdentifier) -> usize {
        self.neighbors_directed(node, petgraph::EdgeDirection::Incoming)
            .count()
    }
}

// SeenNodes stores information about already seen nodes. Due to the nature of
// the data in BFCounter all edges are unique, but they may reuse some already
// seen nodes. To reduce memory usage we only store NodeSlices against which we
// compare new reads. To get the NodeIndex on the PtGraph we need to devise a
// way to map NodeSlice's offset to NodeIndex. Given a NodeSlice we can get
// the index of the node if we account for the nodes in the SEQUENCES that
// are not unique (empty). To do this we store a map of the nodes in SEQUENCES
// in the fixedbitset and then we count unique nodes up to the given NodeSlice.
// This method gives us a working NodeIndex.
type SeenNodes = HashSet<NodeSlice, BuildHash<MetroHash>>;
type ReadsToNodes = HashMap<NodeSlice, NodeIndex, BuildHash<MetroHash>>;

// Algorithm described in the above comment works flawlessly although is slow
// due to the nature of repeatable calling count_ones() on fixedbitset. To fight
// this we store a vector of unique nodes counts per multiple blocks in
// fixedbitset. This means that when we need to sum unique nodes up to the given
// offset we need to sum the megablocks up to the one containing the offset and
// then call count_ones() on the remaining several hundred blocks. This approach
// seems to be VERY performant, as LLVM can vectorize the sum. Below constants
// control how many blocks we account for in a single number in the vector of
// counts.
const BLOCKS_PER_NUMBER: usize = 512;
const NODES_PER_NUMBER: usize = BLOCKS_PER_NUMBER * 32;

// Graph builder which stores information about already seen vertices
// TODO maybe change that to the tagged union?
#[derive(Default)]
struct PtGraphBuilder {
    graph: PtGraph,
    seen_nodes: SeenNodes,
    reads_to_nodes: ReadsToNodes,
    fb: FixedBitSet,
    counts: Vec<Idx>,
}

#[inline]
fn div_rem(x: usize) -> (usize, usize) {
    (x / NODES_PER_NUMBER, x % NODES_PER_NUMBER)
}

impl PtGraphBuilder {
    #[inline]
    fn add_bfc_node(&mut self, mut node: NodeSlice) -> NodeIndex {
        let mut insert = false;
        if let Some(key) = self.seen_nodes.get(&node) {
            // node already in the SEQUENCES
            node = *key;
        }
        else {
            // omit immutable borrow on seen_nodes
            insert = true;
        }
        if insert {
            self.seen_nodes.insert(node);
            self.fb.set(node.offset(), true);
            let block = node.offset() / NODES_PER_NUMBER;
            self.counts[block] += 1;
            self.graph.add_node(())
        }
        else {
            self.get_node_idx(node)
        }
    }

    #[inline]
    fn add_fasta_node(&mut self, node: NodeSlice) -> NodeIndex {
        match self.reads_to_nodes.entry(node) {
            Entry::Occupied(oe) => {
                // node already in the SEQUENCES
                *oe.get()
            }
            Entry::Vacant(ve) => {
                let idx = self.graph.add_node(());
                ve.insert(idx);
                idx
            }
        }
    }

    #[inline]
    fn get_node_idx(&self, node: NodeSlice) -> NodeIndex {
        let off = node.offset();
        let (block_idx, last_block_offset) = div_rem(off);
        let mut idx = self.counts[..block_idx].iter().sum::<Idx>();
        if last_block_offset != 0 {
            idx += self.fb.count_ones(block_idx * NODES_PER_NUMBER..off) as Idx;
        }
        NodeIndex::new(idx as usize)
    }

    #[inline]
    fn add_single_edge_fastaq(&mut self, first_edge: bool, compressed: Vec<u8>,
                              s: &mut NodeIndex, t: &mut NodeIndex) {
        let offset;
        {
            let mut s = SEQUENCES.write();
            offset = s.len();
            s.push(compressed.into_boxed_slice());
        }
        if first_edge {
            let source = NodeSlice::new(2 * offset);
            *s = self.add_fasta_node(source);
        }
        let target = NodeSlice::new(2 * offset + 1);
        *t = self.add_fasta_node(target);
        match self.graph.find_edge(*s, *t) {
            // edge already in the graph, update it's weight
            Some(e) => {
                SEQUENCES.write().pop();
                self.graph.edge_weight_mut(e).expect("This should never fail").1 += 1;
            }
            // insert new edge
            None => {
                self.graph.add_edge(*s, *t, (EdgeSlice::from(NodeSlice::new(2 * offset)), 1));
            }
        }
        *s = *t;
    }

    #[inline]
    fn add_single_edge_bfc(&mut self, compressed_kmer: Vec<u8>, weight: EdgeWeight) {
        let offset;
        {
            let mut s = SEQUENCES.write();
            offset = s.len();
            s.push(compressed_kmer.into_boxed_slice());
        }
        let source = NodeSlice::new(2 * offset);
        let target = NodeSlice::new(2 * offset + 1);
        let s = self.add_bfc_node(source);
        let t = self.add_bfc_node(target);
        self.graph.add_edge(s, t, (EdgeSlice::from(source), weight));
    }
}

impl Init for PtGraph {
    fn init(edge_count: Option<usize>, node_count: Option<usize>, _ft: InputFileType) -> PtGraph {
        let nodes = match node_count {
            Some(n) => n,
            None => 0,
        };
        let edges = match edge_count {
            Some(n) => n,
            None => 0,
        };
        PtGraph::with_capacity(nodes, edges)
    }
}

impl Init for PtGraphBuilder {
    fn init(edge_count: Option<usize>, node_count: Option<usize>, ft: InputFileType)
            -> PtGraphBuilder {
        let nodes = match node_count {
            Some(n) => n,
            None => 0,
        };
        let edges = match edge_count {
            Some(n) => n,
            None => 0,
        };
        match ft {
            InputFileType::Fasta | InputFileType::Fastq => {
                PtGraphBuilder {
                    graph: PtGraph::with_capacity(nodes, edges),
                    seen_nodes:
                        SeenNodes::with_capacity_and_hasher(0, BuildHash::<MetroHash>::default()),
                    reads_to_nodes:
                        ReadsToNodes::with_capacity_and_hasher(nodes,
                                                               BuildHash::<MetroHash>::default()),
                    fb: FixedBitSet::with_capacity(0),
                    counts: Vec::with_capacity(0),
                }
            }
            InputFileType::BFCounter => {
                PtGraphBuilder {
                    graph: PtGraph::with_capacity(nodes, edges),
                    seen_nodes:
                        SeenNodes::with_capacity_and_hasher(nodes,
                                                            BuildHash::<MetroHash>::default()),
                    reads_to_nodes:
                        ReadsToNodes::with_capacity_and_hasher(0, BuildHash::<MetroHash>::default()),
                    // each edge can create 2 nodes, and there are 2 dummy nodes
                    // created by the 0 and 1 offset, used as temporary
                    // placeholders in SEQUENCES. Refer to SEQUENCES
                    // documentation for more information.
                    fb: FixedBitSet::with_capacity(4 * edges + 2),
                    counts:
                        vec![0; ((4 * edges + 2) as f64 / NODES_PER_NUMBER as f64).ceil() as usize],
                }
            }
        }
    }
}

impl Build for PtGraphBuilder {
    #[inline]
    fn add_read_fastaq(&mut self, read: &[u8], reverse_complement: bool) {
        assert!(read.len() as Idx >= unsafe { K_SIZE }, "Read is too short!");
        let mut s = NodeIndex::default();
        let mut t = NodeIndex::default();

        if reverse_complement {
            // because underlying algorithm which adds nodes/edges to the graph
            // relies on the fact that each time we slide the window the old
            // target node becomes the new souce node, we cant insert reverse
            // complement after kmer is compressed, but rather we need to store
            // all reverse complements and add them after original read has been
            // added. What is essentially happening in here is that first all
            // read kmers are added and then all reverse complements are added.
            // Generation of reverse complements is mangled together with
            // compression of read kmers for performance reasons.
            let mut reversed = Vec::with_capacity(read.len() - unsafe { K1_SIZE });
            // let remainder = unsafe{ K1_SIZE } % 4;
            for (cnt, window) in read.windows(unsafe { K_SIZE } as usize).enumerate() {
                // compress k_mer, generate compressed reverse complement of the
                // kmer and store it to add after all kmers for the read are
                // generated
                let (compressed_kmer, rev_compl_compr) = compress_kmer_with_rev_compl(window);
                reversed.push(rev_compl_compr);
                self.add_single_edge_fastaq(cnt == 0, compressed_kmer, &mut s, &mut t);
            }
            // add reverse complements
            let rev = reversed.len() - 1;
            self.add_single_edge_fastaq(true, reversed.remove(rev), &mut s, &mut t);
            for r in reversed.drain(..).rev() {
                self.add_single_edge_fastaq(false, r, &mut s, &mut t);
            }
        }
        else {
            for (cnt, window) in read.windows(unsafe { K_SIZE } as usize).enumerate() {
                let compressed_kmer = compress_kmer(window);
                self.add_single_edge_fastaq(cnt == 0, compressed_kmer, &mut s, &mut t);
            }
        }
    }

    #[inline]
    fn add_read_bfc(&mut self, read: &[u8], weight: EdgeWeight, reverse_complement: bool) {
        assert!(read.len() as Idx >= unsafe { K_SIZE }, "Read is too short!");
        if reverse_complement {
            let (compressed_kmer, rev_compl_compr) = compress_kmer_with_rev_compl(read);
            self.add_single_edge_bfc(compressed_kmer, weight);
            self.add_single_edge_bfc(rev_compl_compr, weight);
        }
        else {
            let compressed_kmer = compress_kmer(read);
            self.add_single_edge_bfc(compressed_kmer, weight);
        }
    }
}

impl Build for PtGraph {
    fn create<P: AsRef<Path>>(path: P, ft: InputFileType, reverse_complement: bool,
                              minimal_weight_threshold: EdgeWeight)
                              -> (Self, usize)
        where Self: Sized {
        let (builder, number_of_read_bytes) =
            PtGraphBuilder::create(path, ft, reverse_complement, minimal_weight_threshold);
        let mut s = SEQUENCES.write();
        for mut e in s.iter_mut().skip(1) {
            let new_box = kmer_to_edge(e).into_boxed_slice();
            *e = new_box;
        }
        (builder.graph, number_of_read_bytes)
    }

    fn add_read_fastaq(&mut self, _read: &[u8], _reverse_complement: bool) {
        unimplemented!();
    }
}
