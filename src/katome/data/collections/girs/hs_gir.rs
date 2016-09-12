//! `HashSet` based Graph's Intermediate Representation
use std::mem;
use std::fmt;
extern crate itertools;
use self::itertools::Itertools;

use asm::assembler::SEQUENCES;
use data::edges::Edges;
use data::read_slice::ReadSlice;
use data::primitives::{K_SIZE, K1_SIZE, Idx};
use data::vertex::Vertex;
use algorithms::builder::Build;
use data::collections::girs::{GIR, Convert};
use data::collections::graphs::pt_graph::{PtGraph, NodeIndex};

use std::collections::HashSet as HS;
use std::hash::BuildHasherDefault;

extern crate metrohash;
use self::metrohash::MetroHash;
// use ::pbr::{ProgressBar};

/// `HashSet` GIR
pub type HsGIR = HS<Box<Vertex>, BuildHasherDefault<MetroHash>>;

impl GIR for HsGIR {}

impl Build for HsGIR {

    /// Add new reads to `HmGIR`, modify weights of existing edges.
    fn add_read(&mut self, read: &[u8]) {
        assert!(read.len() as Idx >= K_SIZE, "Read is too short!");
        let mut ins_counter: Idx = 0;
        let mut current: Box<Vertex>;
        let mut previous_node = Box::new(Vertex::new(ReadSlice::new(0), Edges::default()));
        let mut offset;
        let mut idx = self.len();
        let mut current_idx;
        let mut insert = false;
        for (cnt, window) in read.windows(K1_SIZE as usize).enumerate() {
            let rs = {
                let mut s = unwrap!(SEQUENCES.write(), "Global sequences poisoned :(");
                offset = s.len();
                // append new data to the global vector of sequences
                if ins_counter == 0 || ins_counter > K1_SIZE {
                    // append window to vector
                    s.extend_from_slice(window);
                    ReadSlice::new(offset as Idx)
                }
                else {
                    // append only ins_counter last bytes of window
                    s.extend_from_slice(&window[(K1_SIZE - ins_counter) as usize..]);
                    ReadSlice::new(offset - (K1_SIZE - ins_counter) as Idx)
                }
            };
            current = Box::new(Vertex::new(rs, Edges::empty(idx)));
            if let Some(v) = self.get(&current) {
                // sequence already exists, we should remove redundant bytes from
                // SEQUENCES and update counters
                if ins_counter > 0 {
                    ins_counter += 1;
                }
                unwrap!(SEQUENCES.write()).truncate(offset);
                current_idx = v.edges.idx;
                current = v.clone();
            }
            else {
                insert = true;
                ins_counter = 1;
                current_idx = idx;
                idx += 1;
            }
            if insert {
                self.insert(current.clone());
                insert = false;
            }
            if cnt > 0 {
                let same_node = current == previous_node;
                create_or_modify_edge(&mut previous_node.edges, current_idx);
                if same_node {
                    current = previous_node.clone();
                }
                self.replace(previous_node);
            }
            previous_node = current;
        }
    }
}

/// Create edge if it previously haven't existed, otherwise increase it's weight.
pub fn create_or_modify_edge(edges: &mut Edges, to: Idx) {
    for i in edges.outgoing.iter_mut() {
        if i.0 == to {
            i.1 += 1;
            return;
        }
    }
    let mut out_ = Vec::new();
    out_.extend_from_slice(&edges.outgoing);
    out_.push((to, 1));
    edges.outgoing = out_.into_boxed_slice();
}

impl Convert<HsGIR> for PtGraph {
    fn create_from(h: HsGIR) -> Self {
        let mut graph = PtGraph::default();
        for vertex in h.into_iter() {
            let source = NodeIndex::new(vertex.edges.idx);
            while source.index() >= graph.node_count() {
                graph.add_node(ReadSlice::default());
            }
            for edge in vertex.edges.outgoing.into_iter() {
                while edge.0 >= graph.node_count() {
                    graph.add_node(ReadSlice::default());
                }
                graph.add_edge(source, NodeIndex::new(edge.0), edge.1);
            }
            *unwrap!(graph.node_weight_mut(source)) = vertex.rs.clone();

            // deallocate box such that it does not occupy memory
            let raw = Box::into_raw(vertex);
            deallocate(raw);
        }
        graph
    }
}

// hack to deallocate `Vertex` (avoids unstable `heap` api)
// for more information see https://github.com/rust-lang/rust/issues/27700
fn deallocate<T>(ptr: *mut T) {
    unsafe{ mem::drop(Vec::from_raw_parts(ptr, 0, 1)); }
}

/// Convenience wrapper around `HsGIR`, allows for a custom Debug trait implementation
pub struct DebugHsGIR(pub HsGIR);

fn id<T>(x: T, _: T) -> T { x }

impl fmt::Debug for DebugHsGIR {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0
            .iter()
            .map(|node| {
                node.edges.outgoing
                    .iter()
                    .map(|&e| {
                        let mut sequence = node.rs.name();
                        sequence.push(ReadSlice::new(e.0).last_char());
                        writeln!(f, "sequence {} weight {}", sequence, e.1)
                    })
                    .fold_results((), id)
            })
            .fold_results((), id)
    }
}
