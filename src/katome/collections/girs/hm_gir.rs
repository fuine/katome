//! `HashMap` based Graph's Intermediate Representation

use algorithms::builder::{Build, Init};
use asm::SEQUENCES;
use collections::{Convert, GIR};
use collections::girs::edges::{Edge, Outgoing};
use collections::graphs::pt_graph::{NodeIndex, PtGraph};
use compress::{change_last_char_in_edge, compress_kmer, kmer_to_edge, compress_kmer_with_rev_compl};
use config::InputFileType;
use prelude::{Idx, K_SIZE, K1_SIZE};
use slices::{BasicSlice, EdgeSlice, NodeSlice};
use super::hs_gir::create_or_modify_edge;

use metrohash::MetroHash;
use fixedbitset::FixedBitSet;

use std::collections::HashMap as HM;
use std::collections::hash_map::Entry;
use std::hash::BuildHasherDefault as BuildHash;

/// `HashMap` GIR
pub type HmGIR = HM<NodeSlice, Outgoing, BuildHash<MetroHash>>;

impl GIR for HmGIR {}

impl Init for HmGIR {
    fn init(_edge_count: Option<usize>, node_count: Option<usize>, _ft: InputFileType) -> HmGIR {
        if let Some(nodes) = node_count {
            HmGIR::with_capacity_and_hasher(nodes, BuildHash::<MetroHash>::default())
        }
        else {
            HmGIR::default()
        }
    }
}

impl Build for HmGIR {
    /// Add new reads to `HmGIR`, modify weights of existing edges.
    fn add_read_fastaq(&mut self, read: &[u8], reverse_complement: bool) {
        assert!(read.len() as Idx >= unsafe { K_SIZE }, "Read is too short!");
        let mut s = NodeSlice::default();
        let mut t = NodeSlice::default();
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
                reversed.push((rev_compl_compr, window[window.len() - 1]));
                add_single_edge(self,
                                cnt == 0,
                                compressed_kmer,
                                &mut s,
                                &mut t,
                                window[window.len() - 1]);
            }
            // add reverse complements
            let rev = reversed.len() - 1;
            let last = reversed.remove(rev);
            add_single_edge(self, true, last.0, &mut s, &mut t, last.1);
            for r in reversed.drain(..).rev() {
                add_single_edge(self, false, r.0, &mut s, &mut t, r.1);
            }
        }
        else {
            for (cnt, window) in read.windows(unsafe { K_SIZE } as usize).enumerate() {
                let compressed_kmer = compress_kmer(window);
                add_single_edge(self,
                                cnt == 0,
                                compressed_kmer,
                                &mut s,
                                &mut t,
                                window[window.len() - 1]);
            }
        }
    }
}

#[inline]
fn add_single_edge(gir: &mut HmGIR, first_node: bool, compressed: Vec<u8>,
                   source_node: &mut NodeSlice, target_node: &mut NodeSlice, last_char: u8) {
    let mut insert = false;
    {
        let mut s = SEQUENCES.write();
        s[0] = compressed.into_boxed_slice();
    }
    // insert source on the first pass of the loop
    if first_node {
        *source_node = {
            // get a proper key to the hashmap
            match gir.entry(*source_node) {
                Entry::Occupied(oe) => *oe.key(),
                Entry::Vacant(_) => {
                    // push to vector
                    let mut s = SEQUENCES.write();
                    let tmp = s[0].clone();
                    let offset = s.len();
                    s.push(tmp);
                    insert = true;
                    NodeSlice::new(2 * offset)
                }
            }
        };

        if insert {
            gir.insert(*source_node, Box::new([]));
        }
    }

    *target_node = NodeSlice::new(1);
    *target_node = {
        // get a proper key to the hashmap
        match gir.entry(*target_node) {
            Entry::Occupied(oe) => {
                insert = false;
                *oe.key()
            }
            Entry::Vacant(_) => {
                // push to vector
                let offset = if !insert {
                    let mut s = SEQUENCES.write();
                    let tmp = s[0].clone();
                    s.push(tmp);
                    2 * s.len() - 1
                }
                else {
                    source_node.offset() + 1
                };
                insert = true;
                NodeSlice::new(offset)
            }
        }
    };

    if insert {
        gir.insert(*target_node, Box::new([]));
    }
    let e: &mut Outgoing = unwrap!(gir.get_mut(source_node), "Node disappeared");
    create_or_modify_edge(e, target_node.offset(), last_char);
    *source_node = *target_node;
}


impl Convert<HmGIR> for PtGraph {
    fn create_from(mut gir: HmGIR) -> Self {
        info!("Starting conversion from GIR to graph");
        {
            let mut idx_set: HM<Idx, Idx, BuildHash<MetroHash>> =
                HM::with_capacity_and_hasher(gir.len(), BuildHash::<MetroHash>::default());
            for (cnt, ns) in gir.keys().enumerate() {
                idx_set.insert(ns.offset(), cnt);
            }
            for edges in gir.values_mut() {
                *edges = edges.iter()
                    .cloned()
                    .map(|x| (*unwrap!(idx_set.get(&x.0)), x.1, x.2))
                    .collect::<Vec<Edge>>()
                    .into_boxed_slice();
            }
        }
        let mut graph = PtGraph::default();
        let mut s = SEQUENCES.write();
        let mut fb = FixedBitSet::with_capacity(s.len());
        for (idx, (ns, mut _edges)) in gir.drain().enumerate() {
            let source = NodeIndex::new(idx);
            while source.index() >= graph.node_count() {
                graph.add_node(());
            }
            let id = ns.idx();
            if _edges.is_empty() {
                // clear the underlying box as it will no longer be used. We
                // can't pop it out of the global vector cause it would ruin our
                // existing indices that are already in the graph.
                s[id] = Box::new([]);
                continue;
            }
            // at least one edge going out
            let (target, weight, last_char) = _edges[0];
            let prev = fb.put(id);
            let (slice, tmp) = if prev {
                // this slice uses already taken slot with compressed edge - we
                // can't link them both to the same id
                let new_compressed = change_last_char_in_edge(&s[id], last_char);
                s.push(new_compressed.clone().into_boxed_slice());
                (EdgeSlice::new(s.len() - 1), new_compressed)
            }
            else {
                // first edge slice will be pointing at the original place of source
                // node, next edges will be appended to the global SEQUENCEs after
                // having their last symbol changed
                let tmp = kmer_to_edge(&s[id]);
                s[id] = tmp.clone().into_boxed_slice();
                (EdgeSlice::new(id), tmp)
            };
            while target >= graph.node_count() {
                graph.add_node(());
            }
            graph.add_edge(source, NodeIndex::new(target), (slice, weight));
            for edge in _edges.into_iter().skip(1) {
                while edge.0 >= graph.node_count() {
                    graph.add_node(());
                }
                let new_compressed = change_last_char_in_edge(&tmp, edge.2);
                s.push(new_compressed.into_boxed_slice());
                graph.add_edge(source,
                               NodeIndex::new(edge.0),
                               (EdgeSlice::new(s.len() - 1), edge.1));
            }
            // force drop of the original box in hashmap
            _edges = Box::new([]);
        }
        graph
    }
}
