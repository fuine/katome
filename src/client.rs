extern crate katome;
extern crate pbr;

// use katome::data::sequences::{sequence_to_u64, u64_to_sequence};
use katome::data::input::{read_sequences, as_u8_slice, add_sequence_to_graph};
use katome::data::types::{Graph, K_SIZE};

use pbr::{ProgressBar};

fn main(){
    // let v = read_sequences("***REMOVED***".to_string());
    // let v = read_sequences("***REMOVED***".to_string());
    let v = read_sequences("./test2.fastaq".to_string());
    // let mut iter = v.iter();
    let mut graph: Graph = Graph::with_capacity(v.len());
    // let mut pb = ProgressBar::new(v.len() as u64);
    let mut counter = 0;
    // pb.format("╢▌▌░╟");
    for i in 0..v.len() {
        add_sequence_to_graph(&v[i], &mut graph, K_SIZE, &mut counter);
        // pb.inc();
    }
    println!("\nMap has {} unique keys for {} sequences", graph.len(), counter);
    // for (key, val) in graph.iter() {
    // for val in graph.values() {
        // // println!("{}: {:?}", key.name(), val.weights);
        // println!("{:?}", val.weights);
    // }

}
