use data::types::{VertexId, Weight};

pub type Edge = (VertexId, Weight);
pub type Outgoing = Box<[Edge]>;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Edges{
    pub outgoing: Outgoing,
    pub in_num: usize,                        // data is aligned to 8 bytes in this struct
}

impl Edges {
    pub fn new(to: VertexId) -> Edges {
        Edges {
            outgoing: (vec![(to, 1)]).into_boxed_slice(),
            in_num: 0,
        }
    }
    pub fn empty() -> Edges {
        Edges {
            outgoing: Box::new([]),
            in_num: 0,
        }
    }

    pub fn add_edge(&mut self, to: VertexId) {
        let mut out_ = Vec::new();
        out_.extend_from_slice(&self.outgoing);
        out_.push((to, 1));
        self.outgoing = out_.into_boxed_slice();
    }

    pub fn remove_weak_edges(&mut self, threshold: Weight) {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_edge() {
        let mut e: Edges = Edges::empty();
        assert_eq!(e.outgoing.len(), 0);
        e.add_edge(0);
        assert_eq!(e.outgoing.len(), 1);
    }

    #[test]
    fn remove_weak_edges() {
        let mut e: Edges = Edges::empty();
        assert_eq!(e.outgoing.len(), 0);
        e.add_edge(0);
        e.add_edge(1);
        e.add_edge(2);
        e.outgoing[0].1 += 3;
        assert_eq!(e.outgoing.len(), 3);
        e.remove_weak_edges(2);
        assert_eq!(e.outgoing.len(), 1);
    }

    #[test]
    fn remove_no_edges() {
        let mut e: Edges = Edges::empty();
        assert_eq!(e.outgoing.len(), 0);
        e.remove_weak_edges(2);
        assert_eq!(e.outgoing.len(), 0);
    }
}
