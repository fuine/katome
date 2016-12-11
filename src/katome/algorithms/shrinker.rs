//! Shrink the given graph

use algorithms::pruner::Clean;
use collections::Graph;
use collections::graphs::pt_graph::{PtGraph, NodeIndex, EdgeIndex};

use fixedbitset::FixedBitSet;
use petgraph::Direction::{Incoming, Outgoing};
use petgraph::visit::EdgeRef;

/// Mark graph as shrinkable.
pub trait Shrinkable {
    /// Edge index associated with collection.
    type EdgeIdx;
    /// Node index associated with collection.
    type NodeIdx;
    /// Shrink graph.
    ///
    /// This operation should shrink all straight paths (paths in which all
    /// vertices except source and target of the path have in_degree == out_degree == 1)
    /// It is assumed that after shrinking graph will not have any nodes
    /// connected in this way: s -> x -> ... -> t
    fn shrink(&mut self);
    /// Shrink one single path. This method assumes that `base_edge` argument points
    /// to a valid edge, which target has a single outgoing edge.
    /// Returns index of the shrinked path represented by edge.
    fn shrink_single_path(&mut self, base_edge: Self::EdgeIdx) -> Self::EdgeIdx;
    /// Checks only specified points. Note that these points are not meant to be
    /// the starting points of the straight path, but rather the middle point in
    /// them.
    fn shrink_points(&mut self, possible_inc_points: Vec<Self::NodeIdx>);
}

pub struct ShrinkTraverse {
    fb: FixedBitSet,
    stack: Vec<NodeIndex>,
}

impl ShrinkTraverse {
    pub fn new(graph: &PtGraph) -> ShrinkTraverse {
        let mut v = Vec::new();
        // [TODO]: find perfect circles - 2016-10-30 04:35
        for n in graph.externals(Incoming) {
            v.push(n);
        }
        let len = graph.node_count();
        ShrinkTraverse {
            fb: FixedBitSet::with_capacity(len),
            stack: v,
        }
    }

    pub fn next(&mut self, graph: &PtGraph) -> Option<EdgeIndex> {
        while let Some(&node) = self.stack.last() {
            let mut current_node = node;
            let mut new_ancestor;
            loop {
                new_ancestor = false;
                for e in graph.edges_directed(current_node, Outgoing) {
                    let n = e.target();
                    if self.is_visited(n) {
                        continue;
                    }
                    self.mark_visited(n);
                    if graph.out_degree(n) == 1 && graph.in_degree(n) == 1 {
                        // current_node -> n -> x
                        // that's the only thing we need to start shrinking
                        return Some(e.id());
                    }
                    else {
                        self.stack.push(n);
                        current_node = n;
                        new_ancestor = true;
                        break;
                    }
                }
                if !new_ancestor {
                    self.stack.pop();
                    break;
                }
            }
        }
        None
    }

    fn is_visited(&self, node: NodeIndex) -> bool {
        self.fb.contains(node.index())
    }

    fn mark_visited(&mut self, node: NodeIndex) {
        self.fb.insert(node.index());
    }
}

impl Shrinkable for PtGraph {
    type EdgeIdx = EdgeIndex;
    type NodeIdx = NodeIndex;
    #[inline]
    fn shrink_points(&mut self, possible_inc_points: Vec<Self::NodeIdx>) {
        for n in possible_inc_points {
            if self.out_degree(n) == 1 && self.in_degree(n) == 1 {
                let edge_to_shrink = unwrap!(self.edges_directed(n, Incoming).next()).id();
                self.shrink_single_path(edge_to_shrink);
            }
        }
        self.remove_single_vertices();
    }
    fn shrink(&mut self) {
        debug!("Start shrinking");
        let mut t = ShrinkTraverse::new(self);
        let mut count = 0;
        while let Some(base_edge) = t.next(self) {
            self.shrink_single_path(base_edge);
            count += 1;
        }
        debug!("Shrinked: {}", count);
        self.remove_single_vertices();
        debug!("End shrinking");
    }

    #[inline]
    fn shrink_single_path(&mut self, mut base_edge: EdgeIndex) -> EdgeIndex {
        let (start_node, mut mid_node) = unwrap!(self.edge_endpoints(base_edge));
        loop {
            let next_edge = next_out_edge(self, mid_node);
            let base_edge_weight = *unwrap!(self.edge_weight(base_edge));
            let target = edge_target(self, next_edge);
            // make sure that we remove higher index first to prevent higher index
            // invalidation
            let next_edge_weight = if base_edge.index() < next_edge.index() {
                let tmp = unwrap!(self.remove_edge(next_edge)).0;
                self.remove_edge(base_edge);
                tmp
            }
            else if base_edge == next_edge {
                debug!("Self-referencing loop found");
                return base_edge;
            }
            else {
                self.remove_edge(base_edge);
                unwrap!(self.remove_edge(next_edge)).0
            };

            base_edge_weight.0.merge(next_edge_weight);
            base_edge = self.add_edge(start_node, target, base_edge_weight);
            mid_node = target;
            if self.in_degree(mid_node) != 1 || self.out_degree(mid_node) != 1 ||
               mid_node == start_node {
                return base_edge;
            }
        }
    }
}


#[inline]
fn edge_target(graph: &PtGraph, edge: EdgeIndex) -> NodeIndex {
    unwrap!(graph.edge_endpoints(edge)).1
}

#[inline]
fn next_out_edge(graph: &PtGraph, node: NodeIndex) -> EdgeIndex {
    unwrap!(graph.first_edge(node, Outgoing))
}


#[cfg(test)]
mod tests {
    use ::asm::SEQUENCES;
    use ::asm::lock::LOCK;
    use ::collections::graphs::Graph;
    use ::collections::graphs::pt_graph::{PtGraph, NodeIndex, EdgeIndex};
    use ::compress::compress_edge;
    use ::slices::{EdgeSlice, BasicSlice};
    use super::*;

    macro_rules! setup (
        () => (
            let basic_ = (0..37).into_iter().map(|_| b'A').collect::<Vec<u8>>();
            let mut l1 = basic_.clone();
            l1.extend(b"ACG");
            let c1 = compress_edge(&l1);
            let mut l2 = basic_.clone();
            l2.extend(b"CGT");
            let c2 = compress_edge(&l2);
            let mut l3 = basic_.clone();
            l3.extend(b"GTA");
            let c3 = compress_edge(&l3);
            let mut l4 = basic_.clone();
            l4.extend(b"TAA");
            let c4 = compress_edge(&l4);
            let mut l5 = basic_.clone();
            l5.extend(b"AAC");
            let c5 = compress_edge(&l5);
    // lock here
            let _l = LOCK.lock().unwrap();
            SEQUENCES.write().clear();
            SEQUENCES.write().push(c1.into_boxed_slice());
            SEQUENCES.write().push(c2.into_boxed_slice());
            SEQUENCES.write().push(c3.into_boxed_slice());
            SEQUENCES.write().push(c4.into_boxed_slice());
            SEQUENCES.write().push(c5.into_boxed_slice());
        )
    );

    macro_rules! check_node (
        ($g:ident, $i:expr, $x:expr, $y:expr) => (
            assert_eq!($g.in_degree(NodeIndex::new($i)), $x);
            assert_eq!($g.out_degree(NodeIndex::new($i)), $y);
        )
    );

    macro_rules! check_edge (
        ($g:ident, $i:expr, $o:expr, $s:expr) => (
            let maybe_edge = $g.find_edge(NodeIndex::new($i), NodeIndex::new($o));
            assert!(maybe_edge.is_some());
            let edge = maybe_edge.unwrap();
            let weight = unwrap!($g.edge_weight(edge)).0.name();
            assert_eq!(&weight.as_bytes()[37..], $s.as_bytes());
        )
    );

    #[test]
    fn simplest_case_single_node() {
        setup!();
        let mut g = PtGraph::from_edges(&[(0, 1, (EdgeSlice::new(0), 1)),
                                          (1, 2, (EdgeSlice::new(1), 1))]);
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 2);
        g.shrink_single_path(EdgeIndex::new(0));
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 1);
        check_node!(g, 0, 0, 1);
        check_node!(g, 1, 0, 0);
        check_node!(g, 2, 1, 0);
        check_edge!(g, 0, 2, "ACGT");
    }

    #[test]
    fn simplest_case_traverse() {
        setup!();
        let mut g = PtGraph::from_edges(&[(0, 1, (EdgeSlice::new(0), 1)),
                                          (1, 2, (EdgeSlice::new(1), 1))]);
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 2);
        g.shrink();
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 1);
        check_node!(g, 0, 0, 1);
        check_node!(g, 1, 1, 0);
        check_edge!(g, 0, 1, "ACGT");
    }

    #[test]
    fn simple_circle_single_node() {
        setup!();
        let mut g = PtGraph::from_edges(&[(0, 1, (EdgeSlice::new(0), 1)),
                                          (1, 2, (EdgeSlice::new(1), 1)),
                                          (2, 0, (EdgeSlice::new(2), 1))]);
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 3);
        g.shrink_single_path(EdgeIndex::new(0));
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 1);
        check_node!(g, 0, 1, 1);
        check_node!(g, 1, 0, 0);
        check_node!(g, 2, 0, 0);
        check_edge!(g, 0, 0, "ACGTA");
    }

    /* #[test]
    fn simple_circle_traverse() {
        let c1 = compress_edge(b"ACG");
        let c2 = compress_edge(b"CGT");
        let c3 = compress_edge(b"GTA");
        // lock here
        let _l = LOCK.lock().unwrap();
        SEQUENCES.write().clear();
        SEQUENCES.write().push(c1.into_boxed_slice());
        SEQUENCES.write().push(c2.into_boxed_slice());
        SEQUENCES.write().push(c3.into_boxed_slice());
        let mut g = PtGraph::from_edges(&[
            (0, 1, (EdgeSlice::new(0), 1)), (1, 2, (EdgeSlice::new(1), 1)),
            (2, 0, (EdgeSlice::new(2), 1))
        ]);
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 3);
        shrink(&mut g);
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.node_count(), 1);
        assert_eq!(g.out_degree(&NodeIndex::new(0)), 1);
        assert_eq!(g.in_degree(&NodeIndex::new(0)), 1);
    } */

    #[test]
    fn simple_circle_with_tail_traverse() {
        setup!();
        let mut g = PtGraph::from_edges(&[(0, 1, (EdgeSlice::new(0), 1)),
                                          (1, 2, (EdgeSlice::new(1), 1)),
                                          (2, 3, (EdgeSlice::new(2), 1)),
                                          (3, 1, (EdgeSlice::new(3), 1))]);
        assert_eq!(g.node_count(), 4);
        assert_eq!(g.edge_count(), 4);
        g.shrink();
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 2);
        check_node!(g, 0, 0, 1);
        check_node!(g, 1, 2, 1);
        check_edge!(g, 0, 1, "ACG");
        check_edge!(g, 1, 1, "CGTAA");
    }

    #[test]
    fn two_rays_source_traverse() {
        setup!();
        let mut g = PtGraph::from_edges(&[(0, 1, (EdgeSlice::new(0), 1)),
                                          (1, 2, (EdgeSlice::new(1), 1)),
                                          (0, 3, (EdgeSlice::new(2), 1)),
                                          (3, 4, (EdgeSlice::new(3), 1))]);
        assert_eq!(g.node_count(), 5);
        assert_eq!(g.edge_count(), 4);
        g.shrink();
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 2);
        check_node!(g, 0, 0, 2);
        check_node!(g, 1, 1, 0);
        check_node!(g, 2, 1, 0);
        // old node 4 is now node 1 due to index changes after removal
        check_edge!(g, 0, 2, "ACGT");
        check_edge!(g, 0, 1, "GTAA");
    }

    #[test]
    fn two_rays_sink_traverse() {
        setup!();
        let mut g = PtGraph::from_edges(&[(0, 1, (EdgeSlice::new(0), 1)),
                                          (1, 2, (EdgeSlice::new(1), 1)),
                                          (3, 4, (EdgeSlice::new(2), 1)),
                                          (4, 2, (EdgeSlice::new(3), 1))]);
        assert_eq!(g.node_count(), 5);
        assert_eq!(g.edge_count(), 4);
        g.shrink();
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 2);
        check_node!(g, 0, 0, 1);
        // old node 3 is now node 1
        check_node!(g, 1, 0, 1);
        check_node!(g, 2, 2, 0);
        check_edge!(g, 0, 2, "ACGT");
        check_edge!(g, 1, 2, "GTAA");
    }
}
