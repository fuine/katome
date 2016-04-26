// use katome::data::sequences::{sequence_to_u64, u64_to_sequence};
use ::data::input::{read_sequences};
use ::data::types::{Graph, VecArc, Nodes}; // , VecRcPtr};
// use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use std::thread::sleep;
use std::time::Duration;

// use ::pbr::{ProgressBar};

pub fn make_it_happen(){
    let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
    let mut graph: Graph = Graph::default();
    let mut in_nodes: Nodes = Nodes::default();
    let mut out_nodes: Nodes = Nodes::default();
    let mut saved_counter = 0;
    let mut total_counter = 0;
    read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("./data/test2.txt".to_string(),
                   sequences.clone(), &mut graph,
                   &mut in_nodes, &mut out_nodes,
                   &mut saved_counter, &mut total_counter);
    // println!("Size of vec: {} Size of graph: {}");
    println!("I have the capacity of {} for {} stored sequences", graph.capacity(), graph.len());
    println!("I saved {} out of {} bytes -- {}%", saved_counter, total_counter, saved_counter*100/total_counter);
    graph.shrink_to_fit();
    sequences.borrow_mut().shrink_to_fit();
    println!("In: {:?} Out: {:?}", in_nodes.len(), out_nodes.len());
    // sleep(Duration::new(20, 0));
    // for (key, val) in graph.iter() {
    // for val in graph.values() {
        // println!("{}: {:?} {} {}", key.name(), val.outgoing.iter().fold(Vec::new() as Vec<u64>, |mut vec, ref x| {vec.push(x.1); vec}), val.in_num, val.out_num);
        // println!("{:?}", val.outgoing);
    // }


}
