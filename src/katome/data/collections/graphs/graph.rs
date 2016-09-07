//! Graph's interface.
use algorithms::pruner::Prunable;
use algorithms::standardizer::Standardizable;
// use algorithms::collapser::Collapsable;

/// Graph's interface.
pub trait Graph: Prunable + Standardizable {
    type NodeIdentifier;
    /// Collection storing nodes which are ambiguous nodes.
    ///
    /// Node is considered ambiguous if this condition holds:
    /// ```(in_degree > 1 || out_degree > 1) || (in_degree == 0 && out_degree >= 1)```
    /// where `in_degree` and `out_degree` are counts of incoming and outgoing
    /// edges.
    type AmbiguousNodes;
    /// Finds all ambiguous in the `Graph`.
    fn get_ambiguous_nodes(&self) -> Self::AmbiguousNodes;
    /// Gets number of outgoing edges for the given node.
    fn out_degree(&self, &Self::NodeIdentifier) -> usize;
    /// Gets number of incoming edges for the given node.
    fn in_degree(&self, &Self::NodeIdentifier) -> usize;
}
