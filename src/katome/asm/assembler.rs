// use katome::data::sequences::{sequence_to_u64, u64_to_sequence};
use ::data::input::{read_sequences};
use ::data::types::{Graph, VecArc, Nodes}; //, MyHasher}; // , VecRcPtr};
// use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
// use std::thread::sleep;
// use std::time::Duration;
use std::cmp::max;

// use ::pbr::{ProgressBar};

pub fn make_it_happen(){
    let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
    // let mut graph: Graph = Graph::with_capacity_and_hasher(91008059, MyHasher::default());
    let mut graph: Graph = Graph::default();
    let mut in_nodes: Nodes = Nodes::default();
    let mut out_nodes: Nodes = Nodes::default();
    let mut saved_counter = 0;
    let mut total_counter = 0;
    // sleep(Duration::new(5, 0));
    // read_sequences("***REMOVED***".to_string(),
    read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("./data/test2.txt".to_string(),
                   sequences.clone(), &mut graph,
                   &mut in_nodes, &mut out_nodes,
                   &mut saved_counter, &mut total_counter);
    print_stats(&graph, saved_counter, total_counter);
    graph.shrink_to_fit();
    sequences.borrow_mut().shrink_to_fit();
    println!("In: {:?} Out: {:?}", in_nodes.len(), out_nodes.len());
    // sleep(Duration::new(10, 0));
    // for (key, val) in graph.iter() {
    // for val in graph.values() {
        // println!("{}: {:?} {} {}", key.name(), val.outgoing.iter().fold(Vec::new() as Vec<u64>, |mut vec, ref x| {vec.push(x.1); vec}), val.in_num, val.out_num);
        // println!("{:#02} - {}: {:?} {:?}", key.offset, key.name(),  val.outgoing, val.weights);
    // }


}

fn print_stats(graph: &Graph, saved_counter: usize, total_counter: usize) {
    println!("I have the capacity of {} for {} stored sequences", graph.capacity(), graph.len());
    println!("I saved {} out of {} bytes -- {:.2}%", saved_counter, total_counter, (saved_counter*100) as f64/total_counter as f64);
    println!("Max weight: {}", graph.values().fold(0u16, |mx, val| max(mx, val.outgoing.iter().fold(0u16, |m, v| max(m, v.1)))));
    println!("Max in: {}", graph.values().fold(0u32, |mx, val| max(mx, val.in_num)));
    println!("Max out: {}", graph.values().fold(0usize, |mx, val| max(mx, val.outgoing.len())));
    println!("Avg outgoing: {:.2}", (graph.values().fold(0usize, |mx, val| mx + val.outgoing.len())) as f64 / graph.capacity() as f64);
    let real_in = graph.values().filter(|&val| val.in_num > 1).count();
    let real_out = graph.values().filter(|&val| val.outgoing.len() == 0).count();
    println!("Real in: {} ({:.2}%)", real_in, (real_in*100) as f64 / graph.len() as f64);
    println!("Real out: {} ({:.2}%)", real_out, (real_out*100) as f64 / graph.len() as f64);
}
