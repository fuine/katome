// use katome::data::sequences::{sequence_to_u64, u64_to_sequence};
use ::data::input::{read_sequences};
use ::data::types::{Graph, Sequences, VecArc}; // , VecRcPtr};
// use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;

// use ::pbr::{ProgressBar};

pub fn make_it_happen(){
    let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
    let mut graph: Graph = Graph::new();
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    read_sequences("./data/test2.txt".to_string(),
                   sequences.clone(), &mut graph);
    println!("Number of unique sequences: {}", sequences.borrow().len());
    for (key, val) in graph.iter() {
    // for val in graph.values() {
        println!("{}: {:?}", key.name(), val.outgoing.iter().fold(Vec::new() as Vec<u64>, |mut vec, ref x| {vec.push(x.1); vec}));
        // println!("{:?}", val.outgoing);
    }


}
