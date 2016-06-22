use ::data::types::{Graph, VecArc, Weight};
use ::data::read_slice::ReadSlice;
use ::data::edges::Edge;
use std::collections::hash_map::Entry;

pub fn remove_weak_edges(graph: &mut Graph, vec: VecArc, threshold: Weight) {
    let mut to_remove: Vec<ReadSlice> = vec![];
    for vertex in graph.values_mut() {
        // if edge's weight is lower than threshold
        to_remove.extend(vertex.outgoing
            .iter()
            .filter(|&x| x.1 < threshold)
            .map(|&x| RS!(vec, x.0))
            .collect::<Vec<ReadSlice>>());
        vertex.outgoing = vertex.outgoing
            .iter()
            .cloned()
            .filter(|&x| x.1 >= threshold)
            .collect::<Vec<Edge>>()
            .into_boxed_slice();
    }
    for vertex in to_remove {
        if let Entry::Occupied(mut entry) = graph.entry(vertex) {
            entry.get_mut().in_num -= 1;
        }
    }
    remove_not_connected_vertices(graph);
}

pub fn remove_not_connected_vertices(graph: &mut Graph) {
    let keys_to_remove: Vec<ReadSlice> = graph.iter()
        .filter(|&(_, ref val)| val.outgoing.is_empty() && val.in_num == 0)
        .map(|(key, _)| key.clone())
        .collect();
    for key in keys_to_remove {
        graph.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use ::data::types::{Graph, VecArc, K_SIZE};
use ::data::read_slice::ReadSlice;
use ::data::edges::Edges;
use std::sync::Arc;
use std::cell::RefCell;
use ::rand;
use ::rand::Rng;

    #[test]
    fn simple_weak_edge() {
        let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
        let mut graph: Graph = Graph::default();
        // initialize with random data
        sequences.borrow_mut().extend(rand::thread_rng().gen_ascii_chars().take(K_SIZE+2).collect::<String>().into_bytes().into_iter());
        // add 3 vertices with 2 edges (one weak)
        graph.insert(RS!(sequences, 0), Edges::new(1));
        graph.insert(RS!(sequences, 1), Edges::new(2));
        graph.insert(RS!(sequences, 2), Edges::empty());

        {
            let e: &mut Edges = graph.get_mut(&RS!(sequences, 1)).unwrap();
            e.in_num += 1;
            e.outgoing[0].1 = 100;
        }
        {
            let e: &mut Edges = graph.get_mut(&RS!(sequences, 2)).unwrap();
            e.in_num += 1;
        }
        assert_eq!(graph.len(), 3);
        // call remove_weak_edges
        remove_weak_edges(&mut graph, sequences.clone(), 10);
        // chcek if we have 2 vertices and one edge left
        assert_eq!(graph.len(), 2);
    }

    #[test]
    fn empty_graph() {
        let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
        let mut graph: Graph = Graph::default();
        // initialize with random data
        assert_eq!(graph.len(), 0);
        // call remove_weak_edges
        remove_weak_edges(&mut graph, sequences.clone(), 10);
        // chcek if we have 2 vertices and one edge left
        assert_eq!(graph.len(), 0);
    }

    #[test]
    fn only_strong_edges() {
        let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
        let mut graph: Graph = Graph::default();
        // initialize with random data
        sequences.borrow_mut().extend(rand::thread_rng().gen_ascii_chars().take(K_SIZE+2).collect::<String>().into_bytes().into_iter());
        // add 3 vertices with 2 edges (one weak)
        graph.insert(RS!(sequences, 0), Edges::new(1));
        graph.insert(RS!(sequences, 1), Edges::new(2));
        graph.insert(RS!(sequences, 2), Edges::empty());

        {
            let e: &mut Edges = graph.get_mut(&RS!(sequences, 0)).unwrap();
            e.outgoing[0].1 = 100;
        }
        {
            let e: &mut Edges = graph.get_mut(&RS!(sequences, 1)).unwrap();
            e.in_num += 1;
            e.outgoing[0].1 = 100;
        }
        {
            let e: &mut Edges = graph.get_mut(&RS!(sequences, 2)).unwrap();
            e.in_num += 1;
        }
        assert_eq!(graph.len(), 3);
        // call remove_weak_edges
        remove_weak_edges(&mut graph, sequences.clone(), 10);
        // chcek if we have 2 vertices and one edge left
        assert_eq!(graph.len(), 3);
    }

    #[test]
    fn cycle() {
        let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
        let mut graph: Graph = Graph::default();
        // initialize with random data
        sequences.borrow_mut().extend(rand::thread_rng().gen_ascii_chars().take(K_SIZE+2).collect::<String>().into_bytes().into_iter());
        // add 3 vertices with 2 edges (one weak)
        graph.insert(RS!(sequences, 0), Edges::new(1));
        graph.insert(RS!(sequences, 1), Edges::new(2));
        graph.insert(RS!(sequences, 2), Edges::new(0));

        {
            let e: &mut Edges = graph.get_mut(&RS!(sequences, 0)).unwrap();
            e.in_num += 1;
        }
        {
            let e: &mut Edges = graph.get_mut(&RS!(sequences, 1)).unwrap();
            e.in_num += 1;
        }
        {
            let e: &mut Edges = graph.get_mut(&RS!(sequences, 2)).unwrap();
            e.in_num += 1;
        }
        assert_eq!(graph.len(), 3);
        // call remove_weak_edges
        remove_weak_edges(&mut graph, sequences.clone(), 10);
        // chcek if we have 2 vertices and one edge left
        assert_eq!(graph.len(), 0);
    }
}