//! Representation of single node and its outgoing edges.

use data::edges::Edges;
use slices::NodeSlice;

use std::cmp;
use std::hash;

/// Single node and its outgoing edges.
///
/// Used for serialization/deserialization during `GIR` -> `Graph` conversion
#[derive(Clone)]
pub struct Vertex {
    /// Node's `ReadSlice` representing k-mer.
    pub ns: NodeSlice,
    /// Outgoing edges.
    pub edges: Edges,
}

impl Vertex {
    /// Creates new `Vertex`.
    pub fn new(ns_: NodeSlice, edges_: Edges) -> Vertex {
        Vertex {
            ns: ns_,
            edges: edges_,
        }
    }
}

impl hash::Hash for Vertex {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
        self.ns.hash(state)
    }
}

impl cmp::Eq for Vertex {}

impl cmp::PartialEq for Vertex {
    fn eq(&self, other: &Vertex) -> bool {
        self.ns == other.ns
    }
}

impl cmp::PartialOrd for Vertex {
    fn partial_cmp(&self, other: &Vertex) -> Option<cmp::Ordering> {
        self.ns.partial_cmp(&other.ns)
    }
}

impl cmp::Ord for Vertex {
    fn cmp(&self, other: &Vertex) -> cmp::Ordering {
        self.ns.cmp(&other.ns)
    }
}
