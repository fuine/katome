//! `petgraph` based `Graph`.

use algorithms::builder::{Build, Init, InputFileType};
use asm::SEQUENCES;
use data::collections::graphs::Graph;
use data::compress::{compress_kmer, kmer_to_edge};
use data::primitives::{EdgeWeight, Idx, K_SIZE};
use data::slices::{BasicSlice, EdgeSlice, NodeSlice};

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
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

impl Graph for PtGraph {
    type NodeIdentifier = NodeIndex;
    type AmbiguousNodes = HashSet<NodeIndex>;
    fn get_ambiguous_nodes(&self) -> Self::AmbiguousNodes {
        self.node_indices()
            .filter(|n| {
                let in_degree = self.in_degree(n);
                let out_degree = self.out_degree(n);
                (in_degree > 1 || out_degree > 1) || (in_degree == 0 && out_degree >= 1)
            })
            .collect::<Self::AmbiguousNodes>()
    }

    fn out_degree(&self, node: &Self::NodeIdentifier) -> usize {
        self.neighbors_directed(*node, petgraph::EdgeDirection::Outgoing)
            .count()
    }

    fn in_degree(&self, node: &Self::NodeIdentifier) -> usize {
        self.neighbors_directed(*node, petgraph::EdgeDirection::Incoming)
            .count()
    }
}

type SeenNodes = HashSet<NodeSlice, BuildHash<MetroHash>>;
type ReadsToNodes = HashMap<NodeSlice, NodeIndex, BuildHash<MetroHash>>;

// Graph builder which stores information about already seen vertices
// TODO maybe change that to the tagged union?
#[derive(Default)]
struct PtGraphBuilder {
    graph: PtGraph,
    seen_nodes: SeenNodes,
    reads_to_nodes: ReadsToNodes,
    fb: FixedBitSet,
}

impl PtGraphBuilder {
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
            self.graph.add_node(())
        }
        else {
            self.get_node_idx(node)
        }
    }

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

    fn get_node_idx(&self, node: NodeSlice) -> NodeIndex {
        let off = node.offset();
        NodeIndex::new(self.fb.count_ones(..off))
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
                    fb: FixedBitSet::with_capacity(2 * edges + 1),
                }
            }
        }
    }
}

impl Build for PtGraphBuilder {
    fn add_read_fastaq(&mut self, read: &[u8]) {
        assert!(read.len() as Idx >= K_SIZE, "Read is too short!");
        for window in read.windows(K_SIZE as usize) {
            let compressed_kmer = compress_kmer(window);
            let offset;
            {
                let mut s = SEQUENCES.write();
                offset = s.len();
                s.push(compressed_kmer.into_boxed_slice());
            }
            let source = NodeSlice::new(2 * offset);
            let target = NodeSlice::new(2 * offset + 1);
            let s = self.add_fasta_node(source);
            let t = self.add_fasta_node(target);
            // this omits immutable borrow on graph
            let mut exists = None;
            if let Some(e) = self.graph.find_edge(s, t) {
                exists = Some(e);
            }
            match exists {
                // edge already in the graph, update it's weight
                Some(e) => {
                    SEQUENCES.write().pop();
                    self.graph.edge_weight_mut(e).expect("This should never fail").1 += 1;
                }
                // insert new edge
                None => {
                    self.graph.add_edge(s, t, (EdgeSlice::from(source), 1));
                }
            }
        }
    }

    fn add_read_bfc(&mut self, read: &[u8], weight: EdgeWeight) {
        assert!(read.len() as Idx >= K_SIZE, "Read is too short!");
        let compressed_kmer = compress_kmer(read);
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

impl Build for PtGraph {
    fn create<P: AsRef<Path>>(path: P, ft: InputFileType) -> (Self, usize) where Self: Sized {
        let (builder, number_of_read_bytes) = PtGraphBuilder::create(path, ft);
        let mut s = SEQUENCES.write();
        for mut e in s.iter_mut().skip(1) {
            let new_box = kmer_to_edge(e).into_boxed_slice();
            *e = new_box;
        }
        (builder.graph, number_of_read_bytes)
    }

    fn add_read_fastaq(&mut self, _read: &[u8]) {
        unimplemented!();
    }
}
