use algorithms::pruner::Prunable;
use algorithms::standardizer::Standardizable;
// use algorithms::collapser::Collapsable;

pub trait Graph: Prunable + Standardizable {
    type NodeIdentifier;
    type AmbiguousNodes;
    fn get_ambiguous_nodes(&self) -> Self::AmbiguousNodes;
    fn out_degree(&self, &Self::NodeIdentifier) -> usize;
    fn in_degree(&self, &Self::NodeIdentifier) -> usize;
}
