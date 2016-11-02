//! Create string representation of contigs out of `Graph`.

use algorithms::shrinker::Shrinkable;
use collections::Graph;
use collections::graphs::pt_graph::{EdgeIndex, NodeIndex, PtGraph};
use slices::BasicSlice;

use petgraph::EdgeDirection;
use petgraph::visit::EdgeRef;

/// Collapse `Graph` into `SerializedContigs`.
pub trait Collapsable: Shrinkable {
    /// Collapses `Graph` into `SerializedContigs`.
    fn collapse(mut self) -> SerializedContigs;
}

/// Representation of serialized contig.
pub type SerializedContig = String;
/// Collection of serialized contigs.
pub type SerializedContigs = Vec<String>;

impl Collapsable for PtGraph {
    fn collapse(mut self) -> SerializedContigs {
        let mut contigs: SerializedContigs = vec![];
        loop {
            // ensure that we don't end up with straight paths longer than
            // one edge
            self.shrink();
            if let Some(n) = self.externals(EdgeDirection::Incoming).next() {
                contigs.extend(contigs_from_vertex(&mut self, n));
            }
            else {
                break;
            }
        }
        contigs.retain(|x| !x.is_empty());
        contigs
    }
}

fn contigs_from_vertex(graph: &mut PtGraph, v: NodeIndex) -> SerializedContigs {
    let mut contigs: SerializedContigs = vec![];
    let mut contig: SerializedContig = String::new();
    let mut current_vertex = v;
    let mut current_edge_index;
    let mut single_loop;
    loop {
        single_loop = None;
        let number_of_edges = graph.out_degree(current_vertex);
        if number_of_edges == 0 {
            contigs.push(contig.clone());
            return contigs;
        }
        current_edge_index = unwrap!(graph.first_edge(current_vertex, EdgeDirection::Outgoing));
        match number_of_edges {
            0 => {
                unreachable!();
            }
            1 => {
                if requires_shrink(graph, current_edge_index) {
                    // shrink single path and save newly shrinked path as
                    // current edge. This if ensures that loop-recognition
                    // algorithms can work -- they assume that graph is fully
                    // shrinked.
                    current_edge_index = graph.shrink_single_path(current_edge_index);
                    current_vertex = unwrap!(graph.edge_endpoints(current_edge_index)).0;
                }
                // make sure that we are not dealing with the loopy end
                if self_loop(graph, current_vertex).is_none() {
                    single_loop = simple_loop(graph, current_edge_index);
                }
            }
            2 => {
                if requires_shrink(graph, current_edge_index) {
                    // shrink single path and save newly shrinked path as
                    // current edge. This if ensures that loop-recognition
                    // algorithms can work -- they assume that graph is fully
                    // shrinked.
                    current_edge_index = graph.shrink_single_path(current_edge_index);
                    current_vertex = unwrap!(graph.edge_endpoints(current_edge_index)).0;
                }
                // because we handle simple loops in the match arm for 1 then
                // any vertex with 2 outgoing edges has either a self-loop or
                // is ambiguous
                if let Some(e) = self_loop(graph, current_vertex) {
                    current_edge_index = e;
                }
                else {
                    contigs.push(contig.clone());
                    contig.clear();
                }
            }
            _ => {
                // ambiguous edge
                contigs.push(contig.clone());
                contig.clear();
            }
        }
        if contig.is_empty() {
            contig = unwrap!(graph.edge_weight(current_edge_index)).0.name();
        }
        else {
            contig.push_str(&unwrap!(graph.edge_weight(current_edge_index)).0.remainder());
        }
        let (_, target) = unwrap!(graph.edge_endpoints(current_edge_index));
        if let Some(e) = single_loop {
            contig.push_str(&unwrap!(graph.edge_weight(e)).0.remainder());
            // make sure to possibly remove edges in the right order (petgraph
            // will switch the index of the last edge is anything prior to it is
            // removed)
            if current_edge_index < e {
                decrease_weight(graph, e);
                decrease_weight(graph, current_edge_index);
            }
            else {
                decrease_weight(graph, current_edge_index);
                decrease_weight(graph, e);
            }
        }
        else {
            decrease_weight(graph, current_edge_index);
        }
        current_vertex = target;
    }
}

/// Sometimes when collapser creates contigs and removes edges there might be
/// situation, where we have some paths that are not fully shrinked, i.e. they
/// look like x -> y -> ... This function checks if path starting with given
/// edge requires additional shrinkage.
fn requires_shrink(graph: &PtGraph, edge: EdgeIndex) -> bool {
    let (source, target) = unwrap!(graph.edge_endpoints(edge));
    let in_target = graph.in_degree(target);
    let out_target = graph.out_degree(target);
    in_target == 1 && out_target == 1 && source != target
}

/// Check if node has self-loop.
///
/// Only loops that are allowed look like this:
/// a -> b -> c, b -> b
/// a -> b, b -> b
/// It is assumed that node has 1 or 2 outgoing edges
/// TODO add simple diagram to illustrate
fn self_loop(graph: &PtGraph, node: NodeIndex) -> Option<EdgeIndex> {
    if graph.in_degree(node) > 2 {
        return None;
    }
    for e in graph.edges_directed(node, EdgeDirection::Outgoing) {
        if e.target() == e.source() {
            return Some(e.id());
        }
    }
    None
}

/// Check if node has simple loop.
///
/// Simple loop has only one entrance and one exit.
/// Function does not deal well with self loops.
/// Note that shrinking enforces only two nodes in the loop.
/// Edges source has to have exactly one outgoing edge (functions argument).
/// TODO add dragram
fn simple_loop(graph: &PtGraph, edge: EdgeIndex) -> Option<EdgeIndex> {
    let (source, target) = unwrap!(graph.edge_endpoints(edge));
    let in_source = graph.in_degree(source);
    if in_source == 0 || in_source > 2 {
        return None;
    }
    let in_target = graph.in_degree(target);
    let out_target = graph.out_degree(target);
    // since this isn't self loop these conditions must hold
    // note that shrinking ensures that we won't have situation like this:
    // a -> b -> c, b -> d, d -> b
    if in_target != 1 || out_target != 2 {
        return None;
    }
    for e in graph.edges_directed(target, EdgeDirection::Outgoing) {
        if e.target() == source {
            return Some(e.id());
        }
    }
    None
}

fn decrease_weight(graph: &mut PtGraph, edge: EdgeIndex) {
    {
        let edge_mut = unwrap!(graph.edge_weight_mut(edge),
                               "Trying to decrease weight of non-existent edge");
        edge_mut.1 -= 1;
        if edge_mut.1 > 0 {
            return;
        }
    }
    // weight is equal to zero - edge should be removed
    graph.remove_edge(edge);
}

#[cfg(test)]
mod tests {
    pub use ::asm::SEQUENCES;
    pub use ::asm::lock::LOCK;
    pub use ::collections::graphs::pt_graph::PtGraph;
    pub use ::compress::compress_edge;
    pub use ::prelude::{K1_SIZE, K_SIZE};
    pub use ::slices::{BasicSlice, EdgeSlice};
    pub use std::iter::repeat;
    pub use super::*;

    describe! collapse {
        before_each {
            // global lock on sequences for test
            let _l = LOCK.lock().unwrap();
            let mut name = repeat('A')
                .take(K1_SIZE)
                .collect::<String>();
            let mut second = name.clone();
            name.push_str("TGCT");
            second.push_str("G");
            let c1 = compress_edge(name[..K_SIZE].as_bytes());
            let c2 = compress_edge(name[1..K_SIZE+1].as_bytes());
            let c3 = compress_edge(name[2..K_SIZE+2].as_bytes());
            let c4 = compress_edge(name[3..K_SIZE+3].as_bytes());
            let c5 = compress_edge(second[..K_SIZE].as_bytes());
            {
                let mut seq = SEQUENCES.write();
                seq.clear();
                seq.push(c1.into_boxed_slice());
                seq.push(c2.into_boxed_slice());
                seq.push(c3.into_boxed_slice());
                seq.push(c4.into_boxed_slice());
                seq.push(c5.into_boxed_slice());
            }
            let mut graph: PtGraph = PtGraph::default();
            let _w = graph.add_node(());
            let _x = graph.add_node(());
            assert_eq!(graph.node_count(), 2);
        }

        it "doesn't create any contig" {
            assert_eq!(graph.edge_count(), 0);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 0);
        }

        it "creates one small contig" {
            graph.add_edge(_w, _x, (EdgeSlice::new(0), 1));
            assert_eq!(graph.edge_count(), 1);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            assert_eq!(contigs[0].as_str(), &name[..K_SIZE]);
        }

        it "creates one longer contig" {
            let y = graph.add_node(());
            let z = graph.add_node(());
            graph.add_edge(_w, _x, (EdgeSlice::new(0), 1));
            graph.add_edge(_x, y, (EdgeSlice::new(1), 1));
            graph.add_edge(y, z, (EdgeSlice::new(2), 1));
            assert_eq!(graph.edge_count(), 3);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            assert_eq!(contigs[0].as_str(), &name[..K_SIZE+2]);
        }

        it "creates two contigs" {
            let y = graph.add_node(());
            let z = graph.add_node(());
            graph.add_edge(_w, _x, (EdgeSlice::new(0), 1));
            graph.add_edge(y, z, (EdgeSlice::new(2), 1));
            assert_eq!(graph.edge_count(), 2);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 2);
            assert_eq!(contigs[0].as_str(), &name[..K_SIZE]);
            assert_eq!(contigs[1].as_str(), &name[2..K_SIZE+2]);
        }

        it "creates two longer contigs" {
            let y = graph.add_node(());
            let z = graph.add_node(());
            graph.add_edge(_w, _x, (EdgeSlice::new(0), 2));
            graph.add_edge(_x, y, (EdgeSlice::new(1), 1));
            graph.add_edge(_x, z, (EdgeSlice::new(4), 1));
            assert_eq!(graph.edge_count(), 3);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 3);
            assert_eq!(contigs[0].as_str(), &name[..K_SIZE]);
            assert_eq!(contigs[2].as_str(), &name[..K_SIZE+1]);
            assert_eq!(contigs[1], second);
        }

        it "deals with simple cycle" {
            let y = graph.add_node(());
            let z = graph.add_node(());
            graph.add_edge(_w, _x, (EdgeSlice::new(0), 1));
            graph.add_edge(_x, y, (EdgeSlice::new(1), 1));
            graph.add_edge(y, z, (EdgeSlice::new(2), 1));
            graph.add_edge(z, _x, (EdgeSlice::new(3), 1));
            assert_eq!(graph.edge_count(), 4);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            assert_eq!(contigs[0], name);
        }

        // TODO add more tessts for self_loop and simple_loop
    }
}
