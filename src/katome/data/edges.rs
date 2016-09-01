use data::primitives::{EdgeWeight, Idx};

/// Edges representation in GIR. It saves information about outgoing edges, in which tuples
/// of id and weight indicate a single edge.

/// `Idx` indicates unique id of the endpoint node of the edge, assigned based on the
/// GIR creation order.
pub type Edge = (Idx, EdgeWeight);
pub type Outgoing = Box<[Edge]>;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Edges {
    pub outgoing: Outgoing,
    pub idx: Idx,
}

impl Edges {
    pub fn new(to: Idx, idx_: Idx) -> Edges {
        Edges {
            outgoing: (vec![(to, 1)]).into_boxed_slice(),
            idx: idx_,
        }
    }
    pub fn empty(idx_: Idx) -> Edges {
        Edges {
            outgoing: Box::new([]),
            idx: idx_,
        }
    }

    pub fn add_edge(&mut self, to: Idx) {
        let mut out_ = Vec::new();
        out_.extend_from_slice(&self.outgoing);
        out_.push((to, 1));
        self.outgoing = out_.into_boxed_slice();
    }

    pub fn remove_weak_edges(&mut self, threshold: EdgeWeight) {
        self.outgoing = self.outgoing
            .iter()
            .cloned()
            .filter(|&x| x.1 >= threshold)
            .collect::<Vec<Edge>>()
            .into_boxed_slice();
    }

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
            e.add_edge(0);
            assert_eq!(e.outgoing.len(), 1);
        }

        it "removes weak edges" {
            let mut e: Edges = Edges::empty(1);
            assert_eq!(e.idx, 1);
            assert_eq!(e.outgoing.len(), 0);
            e.add_edge(0);
            e.add_edge(1);
            e.add_edge(2);
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
