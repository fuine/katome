use ::data::types::{Graph, Weight};
// use ::data::read_slice::ReadSlice;
// use ::data::edges::Edge;
// use asm::assembler::{SEQUENCES};
// use std::collections::hash_map::Entry;
use ::algorithms::pruner::remove_single_vertices;

pub fn remove_weak_edges(graph: &mut Graph, threshold: Weight) {
    graph.retain_edges(|g, e| {
        !(*g.edge_weight(e).unwrap() < threshold)
    });
    remove_single_vertices(graph);
}
/*
pub fn remove_not_connected_vertices(graph: &mut Graph) {
    let keys_to_remove: Vec<ReadSlice> = graph.iter()
        .filter(|&(_, ref val)| val.outgoing.is_empty() && val.in_num == 0)
        .map(|(key, _)| key.clone())
        .collect();
    for key in keys_to_remove {
        graph.remove(&key);
    }
} */

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

    // #[test]
    // fn simple_weak_edge() {
        // let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
        // let mut graph: Graph = Graph::default();
        // // initialize with random data
        // sequences.borrow_mut().extend(rand::thread_rng().gen_ascii_chars().take(K_SIZE+2).collect::<String>().into_bytes().into_iter());
        // // add 3 vertices with 2 edges (one weak)
        // let rs1 = RS!(1);
        // let rs2 = RS!(2);
        // graph.insert(RS!(0), Edges::new(1));
        // graph.insert(RS!(1), Edges::new(2));
        // graph.insert(RS!(2), Edges::empty());

        // {
            // let e: &mut Edges = graph.get_mut(&rs1).unwrap();
            // e.in_num += 1;
            // e.outgoing[0].1 = 100;
        // }
        // {
            // let e: &mut Edges = graph.get_mut(&rs2).unwrap();
            // e.in_num += 1;
        // }
        // assert_eq!(graph.len(), 3);
        // // call remove_weak_edges
        // remove_weak_edges(&mut graph, sequences.clone(), 10);
        // // chcek if we have 2 vertices and one edge left
        // assert_eq!(graph.len(), 2);
        // assert_eq!(graph.get(&rs1).unwrap().in_num, 0);
        // assert_eq!(graph.get(&rs2).unwrap().in_num, 1);
    // }

    // #[test]
    // fn empty_graph() {
        // let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
        // let mut graph: Graph = Graph::default();
        // // initialize with random data
        // assert_eq!(graph.len(), 0);
        // // call remove_weak_edges
        // remove_weak_edges(&mut graph, sequences.clone(), 10);
        // // chcek if we have 2 vertices and one edge left
        // assert_eq!(graph.len(), 0);
    // }

    // #[test]
    // fn only_strong_edges() {
        // let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
        // let mut graph: Graph = Graph::default();
        // // initialize with random data
        // sequences.borrow_mut().extend(rand::thread_rng().gen_ascii_chars().take(K_SIZE+2).collect::<String>().into_bytes().into_iter());
        // // add 3 vertices with 2 edges (one weak)
        // graph.insert(RS!(sequences, 0), Edges::new(1));
        // graph.insert(RS!(sequences, 1), Edges::new(2));
        // graph.insert(RS!(sequences, 2), Edges::empty());

        // {
            // let e: &mut Edges = graph.get_mut(&RS!(sequences, 0)).unwrap();
            // e.outgoing[0].1 = 100;
        // }
        // {
            // let e: &mut Edges = graph.get_mut(&RS!(sequences, 1)).unwrap();
            // e.in_num += 1;
            // e.outgoing[0].1 = 100;
        // }
        // {
            // let e: &mut Edges = graph.get_mut(&RS!(sequences, 2)).unwrap();
            // e.in_num += 1;
        // }
        // assert_eq!(graph.len(), 3);
        // // call remove_weak_edges
        // remove_weak_edges(&mut graph, sequences.clone(), 10);
        // // chcek if we have 2 vertices and one edge left
        // assert_eq!(graph.len(), 3);
    // }

    // #[test]
    // fn cycle() {
        // let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
        // let mut graph: Graph = Graph::default();
        // // initialize with random data
        // sequences.borrow_mut().extend(rand::thread_rng().gen_ascii_chars().take(K_SIZE+2).collect::<String>().into_bytes().into_iter());
        // // add 3 vertices with 2 edges (one weak)
        // graph.insert(RS!(sequences, 0), Edges::new(1));
        // graph.insert(RS!(sequences, 1), Edges::new(2));
        // graph.insert(RS!(sequences, 2), Edges::new(0));

        // {
            // let e: &mut Edges = graph.get_mut(&RS!(sequences, 0)).unwrap();
            // e.in_num += 1;
        // }
        // {
            // let e: &mut Edges = graph.get_mut(&RS!(sequences, 1)).unwrap();
            // e.in_num += 1;
        // }
        // {
            // let e: &mut Edges = graph.get_mut(&RS!(sequences, 2)).unwrap();
            // e.in_num += 1;
        // }
        // assert_eq!(graph.len(), 3);
        // // call remove_weak_edges
        // remove_weak_edges(&mut graph, sequences.clone(), 10);
        // // chcek if we have 2 vertices and one edge left
        // assert_eq!(graph.len(), 0);
    // }
}
