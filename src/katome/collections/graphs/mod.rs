//! De Bruijn Graph.
//!
//! `Graph`s support various algorithms for efficient genome assembly. They can
//! be build from the input file or from the `GIR` if it supports convertion
//! into the specified `Graph`.
pub mod pt_graph;

use algorithms::builder::Build;
use algorithms::collapser::Collapsable;
use algorithms::pruner::Prunable;
use algorithms::standardizer::Standardizable;
use stats::{Stats, CollectionStats};

/// Graph's interface.
pub trait Graph
    : Build + Prunable + Standardizable + Collapsable + Stats<CollectionStats> {
    /// Node identifier.
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
