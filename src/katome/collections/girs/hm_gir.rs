//! `HashMap` based Graph's Intermediate Representation

use algorithms::builder::{Build, Init};
use asm::SEQUENCES;
use config::InputFileType;
use collections::{Convert, GIR};
use collections::graphs::pt_graph::{NodeIndex, PtGraph};
use compress::{change_last_char_in_edge, compress_kmer, kmer_to_edge};
use data::edges::{Edge, Outgoing};
use prelude::{Idx, K_SIZE};
use slices::{BasicSlice, EdgeSlice, NodeSlice};

use metrohash::MetroHash;

use std::collections::HashMap as HM;
use std::collections::hash_map::Entry;
use std::hash::BuildHasherDefault as BuildHash;
use super::hs_gir::create_or_modify_edge;

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
    fn add_read_fastaq(&mut self, read: &[u8]) {
        assert!(read.len() as Idx >= K_SIZE, "Read is too short!");
        let mut source_node;
        let mut target_node;
        let mut insert = false;
        for window in read.windows(K_SIZE as usize) {
            {
                let mut s = SEQUENCES.write();
                s[0] = compress_kmer(window).into_boxed_slice();
            }
            source_node = NodeSlice::new(0);
            target_node = NodeSlice::new(1);
            source_node = {
                // get a proper key to the hashmap
                match self.entry(source_node) {
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
                self.insert(source_node, Box::new([]));
            }

            target_node = {
                // get a proper key to the hashmap
                match self.entry(target_node) {
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
                self.insert(target_node, Box::new([]));
                insert = false;
            }
            let e: &mut Outgoing = unwrap!(self.get_mut(&source_node), "Node disappeared");
            create_or_modify_edge(e, target_node.offset(), window[window.len() - 1]);
        }
    }
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
            // first edge slice will be pointing at the original place of source node, next edges will be appended to the global SEQUENCEs after having their last symbol changed
            let tmp = kmer_to_edge(&s[id]);
            s[id] = tmp.clone().into_boxed_slice();
            // at least one edge going out
            let (target, weight, _) = _edges[0];
            while target >= graph.node_count() {
                graph.add_node(());
            }
            graph.add_edge(source, NodeIndex::new(target), (EdgeSlice::new(id), weight));
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
