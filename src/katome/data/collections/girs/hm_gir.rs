//! `HashMap` based Graph's Intermediate Representation
use asm::assembler::SEQUENCES;
use data::edges::Edges;
use data::read_slice::ReadSlice;
use data::primitives::{K_SIZE, K1_SIZE, Idx};
use super::hs_gir::create_or_modify_edge;
use algorithms::builder::Build;
use data::collections::girs::gir::{GIR, Convert};
use data::collections::graphs::pt_graph::{PtGraph, NodeIndex};

use std::collections::HashMap as HM;
use std::collections::hash_map::Entry;
use std::hash::BuildHasherDefault;

extern crate metrohash;
use self::metrohash::MetroHash;

/// `HashMap` GIR
pub type HmGIR = HM<ReadSlice, Edges, BuildHasherDefault<MetroHash>>;


impl GIR for HmGIR {}

impl Build for HmGIR {
    /// Add new reads to `HmGIR`, modify weights of existing edges.
    fn add_read(&mut self, read: &[u8]) {
        assert!(read.len() as Idx >= K_SIZE, "Read is too short!");
        let mut ins_counter: Idx = 0;
        let mut index_counter = unwrap!(SEQUENCES.read(), "Global sequences poisoned :(").len() as Idx;
        let mut tmp_index_counter;
        let mut current: ReadSlice;
        let mut previous_node: ReadSlice = ReadSlice::new(0);
        let mut offset;
        let mut idx = self.len();
        let mut current_idx;
        for (cnt, window) in read.windows(K1_SIZE as usize).enumerate() {
            let from_tmp = {
                let mut s = unwrap!(SEQUENCES.write(), "Global sequences poisoned :(");
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
                        unwrap!(SEQUENCES.write()).truncate(offset);
                        if ins_counter > 0 {
                            ins_counter += 1;
                        }
                        current_idx = oe.get().idx;
                        oe.key().clone()
                    }
                    Entry::Vacant(ve) => {
                        index_counter += tmp_index_counter;
                        ins_counter = 1;
                        current_idx = idx;
                        idx += 1;
                        ve.insert(Edges::empty(current_idx));
                        ReadSlice::new(index_counter)
                    }
                }
            };
            if cnt > 0 {
                // insert current sequence as a member of the previous
                let e: &mut Edges = unwrap!(self.get_mut(&previous_node), "Node disappeared");
                create_or_modify_edge(e, current_idx);
            }
            previous_node = current;
        }
    }
}


impl Convert<HmGIR> for PtGraph {
    fn create_from(gir: HmGIR) -> Self {
        info!("Starting conversion from GIR to graph");
        // get rid of hashes -- we don't need them anymore
        let mut vec = gir.into_iter().collect::<Vec<(ReadSlice, Edges)>>();
        // sort this vector according to indicees of nodes, guaranteeing proper node creation (node
        // indices are created just like we created ours, but iterator over hashmap likely changed the
        // ordering).
        vec.sort_by(|a, b| a.1.idx.cmp(&b.1.idx));
        // create separate representations of nodes and edges
        let (nodes, edges): (Vec<ReadSlice>, Vec<Edges>) = vec.into_iter().unzip();
        let mut graph = PtGraph::default();
        // digest nodes and move them into the Graph
        for (cnt, node) in nodes.into_iter().enumerate() {
            let tmp = graph.add_node(node).index();
            assert_eq!(tmp, cnt);
        }
        for edges_ in edges.into_iter() {
            let idx = edges_.idx;
            for edge in edges_.outgoing.into_iter() {
                graph.add_edge(NodeIndex::new(idx), NodeIndex::new(edge.0), edge.1);
            }
        }
        info!("Conversion ended!");
        graph
    }
}
