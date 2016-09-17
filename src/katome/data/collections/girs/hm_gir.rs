//! `HashMap` based Graph's Intermediate Representation
use asm::SEQUENCES;
use data::edges::{Edge, Outgoing};
use data::read_slice::ReadSlice;
use data::primitives::{K_SIZE, K1_SIZE, Idx};
use super::hs_gir::create_or_modify_edge;
use algorithms::builder::Build;
use data::collections::girs::{GIR, Convert};
use data::collections::graphs::pt_graph::{PtGraph, NodeIndex};

use std::collections::HashMap as HM;
use std::collections::hash_map::Entry;
use std::hash::BuildHasherDefault as BuildHash;

use metrohash::MetroHash;

/// `HashMap` GIR
pub type HmGIR = HM<ReadSlice, Outgoing, BuildHash<MetroHash>>;


impl GIR for HmGIR {}

impl Build for HmGIR {
    /// Add new reads to `HmGIR`, modify weights of existing edges.
    fn add_read(&mut self, read: &[u8]) {
        assert!(read.len() as Idx >= K_SIZE, "Read is too short!");
        let mut ins_counter: Idx = 0;
        let mut index_counter = SEQUENCES.read().len() as Idx;
        let mut tmp_index_counter;
        let mut current: ReadSlice;
        let mut previous_node: ReadSlice = ReadSlice::new(0);
        let mut offset;
        for (cnt, window) in read.windows(K1_SIZE as usize).enumerate() {
            let from_tmp = {
                let mut s = SEQUENCES.write();
                offset = s.len();
                // push to vector
                if ins_counter == 0 {
                    // append window to vector
                    s.extend_from_slice(window);
                    tmp_index_counter = 0;
                    ReadSlice::new(offset as Idx)
                }
                else if ins_counter > K1_SIZE {
                    // append window to vector
                    s.extend_from_slice(window);
                    tmp_index_counter = K1_SIZE;
                    ReadSlice::new(offset as Idx)
                }
                else {
                    // append only ins_counter last bytes of window
                    s.extend_from_slice(&window[(K1_SIZE - ins_counter) as usize..]);
                    tmp_index_counter = ins_counter;
                    ReadSlice::new(offset - (K1_SIZE - ins_counter) as Idx)
                }
            };
            current = {
                // get a proper key to the hashmap
                match self.entry(from_tmp) {
                    Entry::Occupied(oe) => {
                        // remove added window from SEQUENCES
                        SEQUENCES.write().truncate(offset);
                        if ins_counter > 0 {
                            ins_counter += 1;
                        }
                        oe.key().clone()
                    }
                    Entry::Vacant(ve) => {
                        index_counter += tmp_index_counter;
                        ins_counter = 1;
                        ve.insert(Box::new([]));
                        ReadSlice::new(index_counter)
                    }
                }
            };
            if cnt > 0 {
                // insert current sequence as a member of the previous
                let e: &mut Outgoing = unwrap!(self.get_mut(&previous_node), "Node disappeared");
                create_or_modify_edge(e, current.offset);
            }
            previous_node = current;
        }
    }
}


impl Convert<HmGIR> for PtGraph {
    fn create_from(mut gir: HmGIR) -> Self {
        info!("Starting conversion from GIR to graph");
        {
            let mut idx_set: HM<Idx, Idx, BuildHash<MetroHash>> = HM::with_capacity_and_hasher(gir.len(), BuildHash::<MetroHash>::default());
            for (cnt, rs) in gir.keys().enumerate() {
                idx_set.insert(rs.offset, cnt);
            }
            for edges in gir.values_mut() {
                *edges = edges
                    .iter()
                    .cloned()
                    .map(|x| (*unwrap!(idx_set.get(&x.0)), x.1))
                    .collect::<Vec<Edge>>()
                    .into_boxed_slice();
            }
        }
        let mut graph = PtGraph::default();
        for (idx, (rs, mut _edges)) in gir.drain().enumerate() {
            let source = NodeIndex::new(idx);
            while source.index() >= graph.node_count() {
                graph.add_node(ReadSlice::default());
            }
            for edge in _edges.into_iter() {
                while edge.0 >= graph.node_count() {
                    graph.add_node(ReadSlice::default());
                }
                graph.add_edge(source, NodeIndex::new(edge.0), edge.1);
            }
            *unwrap!(graph.node_weight_mut(source)) = rs.clone();
            // force drop of the original box in hashmap
            _edges = Box::new([]);
        }
        graph
    }
}
