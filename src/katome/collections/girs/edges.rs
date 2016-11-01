//! Edges representation in GIR.

use prelude::{EdgeWeight, Idx};

/// Single edge representation.
///
/// `Idx` indicates unique id of the endpoint node of the edge, assigned based on the
/// GIR creation order.
/// Last byte denotes last character in the kmer.
pub type Edge = (Idx, EdgeWeight, u8);
/// Stores information about consecutive edges.
pub type Outgoing = Box<[Edge]>;

/// Edges representation in GIR. It saves information about outgoing edges, in which tuples
/// of id and weight indicate a single edge.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Edges {
    /// Information about outgoing edges, storing targets and weights.
    pub outgoing: Outgoing,
    /// Index of source node for all edges in outgoing.
    pub idx: Idx,
}

impl Edges {
    /// Creates `Edges` with a single `Edge`.
    pub fn new(to: Idx, idx_: Idx, last_char: u8) -> Edges {
        Edges {
            outgoing: (vec![(to, 1, last_char)]).into_boxed_slice(),
            idx: idx_,
        }
    }

    /// Creates empty `Edges` with given starting node.
    pub fn empty(idx_: Idx) -> Edges {
        Edges {
            outgoing: Box::new([]),
            idx: idx_,
        }
    }

    /// Adds edge with the given endpoint.
    pub fn add_edge(&mut self, to: Idx, last_char: u8) {
        let mut out_ = Vec::new();
        out_.extend_from_slice(&self.outgoing);
        out_.push((to, 1, last_char));
        self.outgoing = out_.into_boxed_slice();
    }

    /// Removes edges with weight below threshold.
    pub fn remove_weak_edges(&mut self, threshold: EdgeWeight) {
        self.outgoing = self.outgoing
            .iter()
            .cloned()
            .filter(|&x| x.1 >= threshold)
            .collect::<Vec<Edge>>()
            .into_boxed_slice();
    }

    /// Removes specified edge.
    pub fn remove_edge(&mut self, idx: usize) {
        let mut out_ = Vec::new();
        out_.extend_from_slice(&self.outgoing);
        out_.remove(idx);
        self.outgoing = out_.into_boxed_slice();
    }
}

impl Default for Edges {
    fn default() -> Edges {
        Edges {
            outgoing: Box::new([]),
            idx: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    describe! e {
        it "adds edge" {
            let mut e: Edges = Edges::empty(0);
            assert_eq!(e.idx, 0);
            assert_eq!(e.outgoing.len(), 0);
            e.add_edge(0, b'a');
            assert_eq!(e.outgoing.len(), 1);
        }

        it "removes weak edges" {
            let mut e: Edges = Edges::empty(1);
            assert_eq!(e.idx, 1);
            assert_eq!(e.outgoing.len(), 0);
            e.add_edge(0, b'a');
            e.add_edge(1, b'a');
            e.add_edge(2, b'a');
            e.outgoing[0].1 += 3;
            assert_eq!(e.outgoing.len(), 3);
            e.remove_weak_edges(2);
            assert_eq!(e.outgoing.len(), 1);
        }

        it "removes no edge" {
            let mut e: Edges = Edges::empty(2);
            assert_eq!(e.idx, 2);
            assert_eq!(e.outgoing.len(), 0);
            e.remove_weak_edges(2);
            assert_eq!(e.outgoing.len(), 0);
        }
    }
}
