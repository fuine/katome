//! Representation of single node and its outgoing edges.

use data::read_slice::ReadSlice;
use data::edges::Edges;
use std::hash;
use std::cmp;

/// Single node and its outgoing edges.
///
/// Used for serialization/deserialization during `GIR` -> `Graph` conversion
#[derive(Clone)]
pub struct Vertex {
    /// Node's `ReadSlice` representing k-mer.
    pub rs: ReadSlice,
    /// Outgoing edges.
    pub edges: Edges,
}

impl Vertex {
    /// Creates new `Vertex`.
    pub fn new(rs_: ReadSlice, edges_: Edges) -> Vertex {
        Vertex {
            rs: rs_,
            edges: edges_,
        }
    }
}

impl hash::Hash for Vertex {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
        self.rs.hash(state)
    }
}

impl cmp::Eq for Vertex {}

impl cmp::PartialEq for Vertex {
    fn eq(&self, other: &Vertex) -> bool {
        self.rs == other.rs
    }
}

impl cmp::PartialOrd for Vertex {
    fn partial_cmp(&self, other: &Vertex) -> Option<cmp::Ordering> {
        self.rs.partial_cmp(&other.rs)
    }
}

impl cmp::Ord for Vertex {
    fn cmp(&self, other: &Vertex) -> cmp::Ordering {
        self.rs.cmp(&other.rs)
    }
}
