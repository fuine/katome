use ::petgraph;

use data::read_slice::ReadSlice;
use std::collections::HashSet;
use data::collections::graphs::graph::Graph;
use data::primitives::{EdgeWeight, Idx};

pub type EdgeIndex = petgraph::graph::EdgeIndex<Idx>;
pub type NodeIndex = petgraph::graph::NodeIndex<Idx>;
pub type Node = petgraph::graph::Node<ReadSlice, Idx>;
type Edge_ = petgraph::graph::Edge<EdgeWeight, Idx>;
// FIXME cleverly use type from graph trait ?
pub type PtAmbiguousNodes = HashSet<NodeIndex>;

pub type PtGraph = petgraph::Graph<ReadSlice, EdgeWeight, petgraph::Directed, Idx>;

impl Graph for PtGraph {
    type NodeIdentifier = NodeIndex;
    type AmbiguousNodes = PtAmbiguousNodes;
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
