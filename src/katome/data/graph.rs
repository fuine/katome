use ::petgraph;

use std::sync::{Arc, RwLock};
use std::collections::HashSet;
use data::read_slice::ReadSlice;

pub type Idx = usize;
pub type EdgeIndex = petgraph::graph::EdgeIndex<Idx>;
pub type NodeIndex = petgraph::graph::NodeIndex<Idx>;
pub type Node = petgraph::graph::Node<ReadSlice, Idx>;
pub type EdgeWeight = u16;
pub const K_SIZE: Idx = 40;
pub const WEAK_EDGE_THRESHOLD: EdgeWeight = 4;

pub type Sequences = Vec<u8>;
pub type VecArc = Arc<RwLock<Sequences>>;
pub type Graph = petgraph::Graph<ReadSlice, EdgeWeight, petgraph::Directed, Idx>;


pub type AmbiguousNodes = HashSet<NodeIndex>;

pub fn get_ambiguous_nodes(graph: &Graph) -> AmbiguousNodes {
    graph.node_indices()
        .filter(|&n| {
            let in_degree = in_degree(graph, n);
            let out_degree = out_degree(graph, n);
            (in_degree > 1 || out_degree > 1) || (in_degree == 0 && out_degree >= 1)
        })
        .collect::<AmbiguousNodes>()
}

pub fn out_degree(graph: &Graph, node: NodeIndex) -> usize {
    graph.neighbors_directed(node, petgraph::EdgeDirection::Outgoing)
        .count()
}

pub fn in_degree(graph: &Graph, node: NodeIndex) -> usize {
    graph.neighbors_directed(node, petgraph::EdgeDirection::Incoming)
        .count()
}
