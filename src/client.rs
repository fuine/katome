extern crate katome;
extern crate pbr;

// use katome::data::sequences::{sequence_to_u64, u64_to_sequence};
use katome::data::input::{read_sequences, as_u8_slice, add_sequence_to_graph};
use katome::data::types::{Graph, Sequences, memy};

use pbr::{ProgressBar};

fn main(){
    // let v = read_sequences("***REMOVED***".to_string());
    // let v = read_sequences("***REMOVED***".to_string());
    let mut sequences: Sequences = Vec::new();
    let mut graph: Graph = Graph::new();
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    // read_sequences("***REMOVED***".to_string(),
    read_sequences("./test3.fastaq".to_string(),
                   &mut sequences, &mut graph);
    // let v = read_sequences("./test2.txt".to_string());
    // println!("{}G", memy(v.len(), v[0].len()));
    // let mut pb = ProgressBar::new(v.len() as u64);
    // let mut counter = 0;
    // pb.format("╢▌▌░╟");
    // for i in 0..v.len() {
        // add_sequence_to_graph(&v[i], &mut graph, K_SIZE, &mut counter);
        // if i % 10000 == 0 {
            // println!("{}: {}G", graph.len(), memy(graph.len()));
        // }
        // pb.inc();
    // }
    println!("\nMap has {} unique keys for sequences", graph.len());
    // for (key, val) in graph.iter() {
    // for val in graph.values() {
        // println!("{}: {:?}", key.name(), val.outgoing.iter().fold(Vec::new() as Vec<u64>, |mut vec, &x| {vec.push(x.1); vec}));
        // println!("{:?}", val.outgoing);
    // }

}


// create whole vector with multiple workers
// map hashset slices to raw pointers

