//! `HashSet` based Graph's Intermediate Representation
extern crate itertools;

use algorithms::builder::{Build, Init};
use asm::SEQUENCES;
use collections::{Convert, GIR};
use collections::graphs::pt_graph::{NodeIndex, PtGraph};
use compress::{change_last_char_in_edge, compress_kmer, kmer_to_edge};
use data::edges::{Edges, Outgoing};
use prelude::{Idx, K_SIZE};
use slices::{BasicSlice, EdgeSlice, NodeSlice};
use data::vertex::Vertex;

use metrohash::MetroHash;
use self::itertools::Itertools;

use std::collections::HashSet as HS;
use std::fmt;
use std::hash::BuildHasherDefault;
use std::mem;

/// `HashSet` GIR
pub type HsGIR = HS<Box<Vertex>, BuildHasherDefault<MetroHash>>;

impl GIR for HsGIR {}

impl Init for HsGIR {}

impl Build for HsGIR {
    /// Add new reads to `HmGIR`, modify weights of existing edges.
    fn add_read_fastaq(&mut self, read: &[u8]) {
        assert!(read.len() as Idx >= K_SIZE, "Read is too short!");
        let mut source_vert: Box<Vertex>;
        let mut target_vert: Box<Vertex>;
        let mut idx = self.len();
        let mut insert = false;

        for window in read.windows(K_SIZE as usize) {
            {
                let mut s = SEQUENCES.write();
                s[0] = compress_kmer(window).into_boxed_slice();
            }
            source_vert = Box::new(Vertex::new(NodeSlice::new(0), Edges::empty(idx)));
            target_vert = Box::new(Vertex::new(NodeSlice::new(1), Edges::empty(idx + 1)));
            if let Some(v) = self.get(&source_vert) {
                source_vert = v.clone();
            }
            else {
                let mut s = SEQUENCES.write();
                let tmp = s[0].clone();
                source_vert.ns = NodeSlice::new(2 * s.len());
                s.push(tmp);
                insert = true;
                idx += 1;
            }
            if insert {
                self.insert(source_vert.clone());
            }

            if let Some(v) = self.get(&target_vert) {
                target_vert = v.clone();
                insert = false;
            }
            else {
                let offset = if !insert {
                    let mut s = SEQUENCES.write();
                    let tmp = s[0].clone();
                    s.push(tmp);
                    2 * s.len() - 1
                }
                else {
                    source_vert.ns.offset() + 1
                };
                target_vert.ns = NodeSlice::new(offset);
                insert = true;
                target_vert.edges.idx = idx;
                idx += 1;
            }
            if insert {
                self.insert(target_vert.clone());
                insert = false;
            }
            create_or_modify_edge(&mut source_vert.edges.outgoing,
                                  target_vert.edges.idx,
                                  window[window.len() - 1]);
            self.replace(source_vert);
        }
    }
}

/// Create edge if it previously haven't existed, otherwise increase it's weight.
pub fn create_or_modify_edge(edges: &mut Outgoing, to: Idx, last_char: u8) {
    for i in edges.iter_mut() {
        if i.0 == to {
            i.1 += 1;
            return;
        }
    }
    let mut out_ = Vec::new();
    out_.extend_from_slice(edges);
    out_.push((to, 1, last_char));
    *edges = out_.into_boxed_slice();
}

impl Convert<HsGIR> for PtGraph {
    fn create_from(mut h: HsGIR) -> Self {
        let mut graph = PtGraph::default();
        let mut s = SEQUENCES.write();
        for vertex in h.drain() {
            let source = NodeIndex::new(vertex.edges.idx);
            while source.index() >= graph.node_count() {
                graph.add_node(());
            }
            let id = vertex.ns.idx();
            let out_len = vertex.edges.outgoing.len();
            if out_len == 0 {
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
            let (target, weight, _) = vertex.edges.outgoing[0];
            while target >= graph.node_count() {
                graph.add_node(());
            }
            graph.add_edge(source, NodeIndex::new(target), (EdgeSlice::new(id), weight));
            if out_len > 1 {
                for edge in &vertex.edges.outgoing[1..] {
                    while edge.0 >= graph.node_count() {
                        graph.add_node(());
                    }
                    let new_compressed = change_last_char_in_edge(&tmp, edge.2);
                    s.push(new_compressed.into_boxed_slice());
                    graph.add_edge(source,
                                   NodeIndex::new(edge.0),
                                   (EdgeSlice::new(s.len() - 1), edge.1));
                }
            }
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
    unsafe {
        mem::drop(Vec::from_raw_parts(ptr, 0, 1));
    }
}

/// Convenience wrapper around `HsGIR`, allows for a custom Debug trait implementation
pub struct DebugHsGIR(pub HsGIR);

fn id<T>(x: T, _: T) -> T {
    x
}

impl fmt::Debug for DebugHsGIR {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0
            .iter()
            .map(|node| {
                node.edges
                    .outgoing
                    .iter()
                    .map(|&e| {
                        let mut sequence = node.ns.name();
                        sequence.push(NodeSlice::new(e.0).last_char());
                        writeln!(f, "sequence {} weight {}", sequence, e.1)
                    })
                    .fold_results((), id)
            })
            .fold_results((), id)
    }
}
