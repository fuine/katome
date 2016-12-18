//! Various algorithms for graph pruning - removing unnecessary vertices/edges.

use asm::SEQUENCES;
use collections::HmGIR;
use collections::girs::edges::Edge;
use collections::graphs::Graph;
use collections::graphs::pt_graph::{EdgeIndex, Node, NodeIndex, PtGraph};
use compress::{compress_node, encode_fasta_symbol};
use prelude::{EdgeWeight, K1_SIZE, K_SIZE};
use slices::{BasicSlice, NodeSlice};

use petgraph::EdgeDirection;

use std::collections::hash_map::Entry;
use std::iter;
use std::slice;
use std::vec::Drain;


/// Describes prunable structure in the sense of genome assembly
pub trait Prunable: Clean {
    /// Remove all input and output dead paths
    fn remove_dead_paths(&mut self);
}

/// A trait for keeping the graph clean.
/// It keeps simple functions used for basic graph cleanups
pub trait Clean {
    /// Remove vertives without any edges.
    fn remove_single_vertices(&mut self);
    /// Remove edges with weight below threshold.
    fn remove_weak_edges(&mut self, threshold: EdgeWeight);
}

impl Prunable for PtGraph {
    fn remove_dead_paths(&mut self) {
        info!("Starting graph pruning");
        let mut to_remove: Vec<EdgeIndex> = vec![];
        loop {
            trace!("Detected {} input/output vertices",
                   Externals::new(self.raw_nodes().iter().enumerate()).count());
            // analyze found input/output vertices
            let mut path_check_vec = vec![];
            for v in Externals::new(self.raw_nodes().iter().enumerate()) {
                // sort into output and input paths
                match v {
                    VertexType::Input(v_) => {
                        // decide whether or not vertex is in the dead path
                        check_dead_path(self,
                                        v_,
                                        EdgeDirection::Incoming,
                                        EdgeDirection::Outgoing,
                                        &mut path_check_vec);
                        if !path_check_vec.is_empty() {
                            to_remove.extend(path_check_vec.drain(..));
                        }
                    }
                    VertexType::Output(v_) => {
                        check_dead_path(self,
                                        v_,
                                        EdgeDirection::Outgoing,
                                        EdgeDirection::Incoming,
                                        &mut path_check_vec);
                        if !path_check_vec.is_empty() {
                            to_remove.extend(path_check_vec.drain(..));
                        }
                    }
                }
            }
            // if there are no dead paths left pruning is done
            if to_remove.is_empty() {
                info!("Graph is pruned");
                return;
            }
            // reverse sort edge indices such that removal won't cause any troubles with swapped
            // edge indices (see `petgraph`'s explanation of `remove_edge`)
            to_remove.sort_by(|a, b| b.cmp(a));
            remove_paths(self, to_remove.drain(..));
        }
    }
}

impl Clean for PtGraph {
    fn remove_single_vertices(&mut self) {
        self.retain_nodes(|g, n| g.neighbors_undirected(n).next().is_some());
    }

    fn remove_weak_edges(&mut self, threshold: EdgeWeight) {
        self.retain_edges(|g, e| unwrap!(g.edge_weight(e)).1 >= threshold);
        self.remove_single_vertices();
    }
}

impl Clean for HmGIR {
    fn remove_single_vertices(&mut self) {
        let mut keys_to_remove: Vec<NodeSlice> = self.iter()
            .filter(|&(_, val)| val.is_empty())
            .map(|(key, _)| *key)
            .collect();
        keys_to_remove = keys_to_remove.into_iter()
            .filter(|x| !has_incoming_edges(self, x))
            .collect();
        for key in keys_to_remove {
            self.remove(&key);
        }
    }

    fn remove_weak_edges(&mut self, threshold: EdgeWeight) {
        for edges in self.values_mut() {
            *edges = edges.iter()
                .cloned()
                .filter(|&x| x.1 >= threshold)
                .collect::<Vec<Edge>>()
                .into_boxed_slice();
        }
        self.remove_single_vertices();
    }
}

/// Utility function which gets us every possible incoming edge.
/// because of memory savings we do not hold an array of incoming edges,
/// instead we will exploit the idea behind sequencing genome, namely
/// common bytes for each sequence.
/// WARNING: this may or may not be optimal if we follow the fasta standard
/// but should be sufficiently faster for just 5 characters we use at the moment
fn has_incoming_edges(gir: &mut HmGIR, node: &NodeSlice) -> bool {
    let mut output = false;
    // copy current sequence to register
    let mut vec = node.byte_name();
    // shift the register one character to the right
    vec.truncate(unsafe { K1_SIZE } - 1);
    vec.insert(0, b'A');
    let mut v = Vec::new();
    compress_node(&vec, &mut v);
    SEQUENCES.write()[0] = v.into_boxed_slice();
    let mask = 0b00111111u8;
    // try to bruteforce by inserting all possible characters: ACTGN
    let tmp_ns = NodeSlice::new(0);
    for chr in &[b'A', b'C', b'T', b'G'] {
        {
            let mut s = SEQUENCES.write();
            s[0][0] &= mask;
            s[0][0] |= encode_fasta_symbol(*chr, 0u8) << 6;
        }
        // dummy read slice used to check if we can find it in the gir
        if let Entry::Occupied(e) = gir.entry(tmp_ns) {
            // if we got any hits check if our vertex is in the outgoing
            if e.get().iter().any(|&x| x.0 == node.offset()) {
                output = true;
                break;
            }
        }
    }
    output
}

enum VertexType<T> {
    Input(T),
    Output(T),
}

/// Iterator yielding vertices which either have no incoming or outgoing edges.
struct Externals<'a> {
    iter: iter::Enumerate<slice::Iter<'a, Node>>,
}

impl<'a> Externals<'a> {
    fn new(iter_: iter::Enumerate<slice::Iter<'a, Node>>) -> Externals {
        Externals { iter: iter_ }
    }
}

impl<'a> Iterator for Externals<'a> {
    type Item = VertexType<NodeIndex>;
    fn next(&mut self) -> Option<VertexType<NodeIndex>> {
        loop {
            match self.iter.next() {
                None => return None,
                Some((index, node)) => {
                    if node.next_edge(EdgeDirection::Incoming) == EdgeIndex::end() {
                        return Some(VertexType::Input(NodeIndex::new(index)));
                    }
                    else if node.next_edge(EdgeDirection::Outgoing) == EdgeIndex::end() {
                        return Some(VertexType::Output(NodeIndex::new(index)));
                    }
                    else {
                        continue;
                    }
                }
            }
        }
    }
}

/// Remove dead input path.
#[inline]
fn remove_paths(graph: &mut PtGraph, to_remove: Drain<EdgeIndex>) {
    trace!("Removing {} dead paths", to_remove.len());
    for e in to_remove {
        let edgepoints = graph.edge_endpoints(e);
        graph.remove_edge(e);
        // guarantee that removal of first node won't change the index of the
        // second node
        if let Some(e) = edgepoints {
            if e.0 < e.1 {
                remove_single_node(graph, e.1);
                remove_single_node(graph, e.0);
            }
            else {
                remove_single_node(graph, e.0);
                remove_single_node(graph, e.1);
            }
        }
    }
}

/// Remove node if it's single.
#[inline]
fn remove_single_node(graph: &mut PtGraph, node: NodeIndex) {
    if graph.in_degree(node) == 0 && graph.out_degree(node) == 0 {
        graph.remove_node(node);
    }
}

/// Check if vertex initializes a dead path.
#[inline]
fn check_dead_path(graph: &PtGraph, vertex: NodeIndex, first_direction: EdgeDirection,
                   second_direction: EdgeDirection, output_vec: &mut Vec<EdgeIndex>) {
    let mut current_vertex = vertex;
    let mut cnt = 0;
    loop {
        cnt += 1;
        if cnt >= 2 * (unsafe { K_SIZE }) {
            // this path is not dead
            output_vec.clear();
            return;
        }
        // this line lets us check outgoing once, without the need to iterate twice
        let next_edge = graph.first_edge(current_vertex, second_direction);
        if let Some(e) = next_edge {
            // add vertex to path
            output_vec.push(e);
            // move to the next vertex in path
            current_vertex = unwrap!(graph.edge_endpoints(e)).1;
        }
        // if vertex has no outgoing edges
        else {
            return;
        }
        if graph.neighbors_directed(current_vertex, first_direction).nth(2).is_some() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_variables)]
    pub use ::collections::graphs::pt_graph::{PtGraph, EdgeIndex};
    pub use ::slices::EdgeSlice;
    pub use super::*;

    #[test]
    fn prunes_single_graph() {
        let mut graph: PtGraph = PtGraph::default();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        graph.remove_weak_edges(10);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    macro_rules! setup {
        ($g:ident, $x:ident, $y:ident, $z:ident) => {
            let mut $g: PtGraph = PtGraph::default();
            let $x = $g.add_node(());
            let $y = $g.add_node(());
            let $z = $g.add_node(());
            assert_eq!($g.node_count(), 3);
        }
    }

    mod remove_weak_edges {
        use super::*;
        #[test]
        fn prunes_single_weak_edge() {
            setup!(graph, x, y, z);
            graph.add_edge(x, y, (EdgeSlice::default(), 100));
            graph.add_edge(y, z, (EdgeSlice::default(), 1));
            assert_eq!(graph.edge_count(), 2);
            graph.remove_weak_edges(10);
            assert_eq!(graph.node_count(), 2);
            assert_eq!(graph.edge_count(), 1);
        }

        #[test]
        fn prunes_single_weak_edge_and_no_nodes() {
            setup!(graph, x, y, z);
            let w = graph.add_node(());
            graph.add_edge(x, y, (EdgeSlice::default(), 100));
            graph.add_edge(y, z, (EdgeSlice::default(), 1));
            graph.add_edge(z, w, (EdgeSlice::default(), 100));
            assert_eq!(graph.edge_count(), 3);
            graph.remove_weak_edges(10);
            assert_eq!(graph.node_count(), 4);
            assert_eq!(graph.edge_count(), 2);
        }

        #[test]
        fn prunes_strong_edges() {
            setup!(graph, x, y, z);
            graph.add_edge(x, y, (EdgeSlice::default(), 100));
            graph.add_edge(y, z, (EdgeSlice::default(), 100));
            assert_eq!(graph.edge_count(), 2);
            graph.remove_weak_edges(10);
            assert_eq!(graph.node_count(), 3);
            assert_eq!(graph.edge_count(), 2);
        }

        #[test]
        fn prunes_cycle() {
            setup!(graph, x, y, z);
            graph.add_edge(x, y, (EdgeSlice::default(), 1));
            graph.add_edge(y, z, (EdgeSlice::default(), 1));
            graph.add_edge(z, x, (EdgeSlice::default(), 1));
            assert_eq!(graph.edge_count(), 3);
            graph.remove_weak_edges(10);
            assert_eq!(graph.node_count(), 0);
            assert_eq!(graph.edge_count(), 0);
        }
    }

    mod remove_single_vertices {
        use super::*;
        #[test]
        fn doesnt_remove_vertices() {
            setup!(graph, x, y, z);
            graph.add_edge(x, y, (EdgeSlice::default(), 100));
            graph.add_edge(y, z, (EdgeSlice::default(), 1));
            assert_eq!(graph.edge_count(), 2);
            graph.remove_single_vertices();
            assert_eq!(graph.node_count(), 3);
            assert_eq!(graph.edge_count(), 2);
        }

        #[test]
        fn removes_one_vertex() {
            setup!(graph, x, y, z);
            graph.add_edge(x, y, (EdgeSlice::default(), 100));
            assert_eq!(graph.edge_count(), 1);
            graph.remove_single_vertices();
            assert_eq!(graph.node_count(), 2);
            assert_eq!(graph.edge_count(), 1);
        }

        #[test]
        fn removes_two_vertices() {
            setup!(graph, x, y, z);
            graph.add_edge(x, x, (EdgeSlice::default(), 100));
            assert_eq!(graph.edge_count(), 1);
            graph.remove_single_vertices();
            assert_eq!(graph.node_count(), 1);
            assert_eq!(graph.edge_count(), 1);
        }

        #[test]
        fn removes_all_vertices() {
            setup!(graph, x, y, z);
            assert_eq!(graph.edge_count(), 0);
            graph.remove_single_vertices();
            assert_eq!(graph.node_count(), 0);
            assert_eq!(graph.edge_count(), 0);
        }

        #[test]
        fn removes_after_removal_of_edge() {
            setup!(graph, x, y, z);
            graph.add_edge(x, y, (EdgeSlice::default(), 100));
            graph.add_edge(y, z, (EdgeSlice::default(), 1));
            assert_eq!(graph.edge_count(), 2);
            assert_eq!(graph.node_count(), 3);
            graph.remove_edge(EdgeIndex::new(1));
            assert_eq!(graph.edge_count(), 1);
            assert_eq!(graph.node_count(), 3);
            graph.remove_single_vertices();
            assert_eq!(graph.node_count(), 2);
            assert_eq!(graph.edge_count(), 1);
        }
    }
}
