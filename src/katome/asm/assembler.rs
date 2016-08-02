use ::data::input::{read_sequences};
use ::data::types::{Graph, VecArc, Weight};
use ::algorithms::pruner::{Pruner};
// use ::algorithms::hardener::remove_weak_edges;
use ::algorithms::standarizer::standarize_edges;
use ::algorithms::euler::euler_paths;
// use ::algorithms::collapser::collapse;
use std::sync::{Arc, RwLock};
use std::iter::repeat;
// use std::cmp::max;
// use std::collections::BTreeSet;
use ::petgraph::EdgeDirection;


lazy_static! {
    pub static ref SEQUENCES: VecArc = Arc::new(RwLock::new(Vec::new()));
}

#[allow(unused_variables)]
pub fn assemble(input: String, output: String, original_genome_length: usize, minimal_weight_threshold: usize) {
    info!("Starting assembler!");
    warn!("test");
    error!("test2");
    // let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
    // let mut graph: Graph = Graph::with_capacity_and_hasher(91008059, MyHasher::default());
    let mut graph: Graph = Graph::default();
    let mut saved_counter = 0;
    let mut number_of_reads: usize = 0;
    let mut number_of_read_bytes = 0;
    read_sequences(input, &mut graph,
                   &mut saved_counter, &mut number_of_read_bytes,
                   &mut number_of_reads);
    print_stats_with_savings(&graph, saved_counter, number_of_read_bytes);
    // // sequences.borrow_mut().shrink_to_fit();
    // remove_weak_edges(&mut graph, WEAK_EDGE_THRESHOLD);
    // print_stats_with_savings(&graph, saved_counter, number_of_read_bytes);
    {
        Pruner::new(&mut graph, ).prune_graph();
    }
    println!("First pruning.");
    print_stats_with_savings(&graph, saved_counter, number_of_read_bytes);
    // graph.shrink_to_fit();
    // let average_read_length: f64 = number_of_read_bytes as f64 / number_of_reads as f64;
    println!("Standarizing");
    standarize_edges(&mut graph, original_genome_length, minimal_weight_threshold as Weight);
    print_stats_with_savings(&graph, saved_counter, number_of_read_bytes);
    println!("Second pruning");
    {
        Pruner::new(&mut graph).prune_graph();
    }
    print_stats_with_savings(&graph, saved_counter, number_of_read_bytes);
    // print_stats_with_savings(&graph, saved_counter, number_of_read_bytes);
    println!("Collapsing!");
    let contigs = euler_paths(&mut graph);
    println!("I created {} contigs", contigs.len());
    // collapse(&mut graph, output);
    info!("All done!");
}

/*
#[allow(dead_code)]
fn check_graph_consistency(graph: &Graph) -> bool {
    let inputs = graph.iter()
        .filter(|&(_, val)| val.in_num == 0)
        .map(|(key, _)| {
            key.offset
        })
        .collect::<BTreeSet<VertexId>>();
    for val in graph.values() {
        for &(k, _) in val.outgoing.iter() {
            if inputs.contains(&k) {
                return false
            }
        }
    }
    true
}
*/

pub fn print_stats_with_savings(graph: &Graph, saved_counter: usize, number_of_read_bytes: usize) {
    println!("I saved {} out of {} bytes -- {:.2}%", saved_counter, number_of_read_bytes, (saved_counter*100) as f64/number_of_read_bytes as f64);
    print_stats(graph);
}

pub fn print_stats(graph: &Graph) {
    println!("I have the capacity of {:?} for {} nodes and {} edges", graph.capacity(), graph.node_count(), graph.edge_count());
    // println!("Max weight: {}", graph.values().fold(0u16, |mx, val| max(mx, val.outgoing.iter().fold(0u16, |m, v| max(m, v.1)))));
    println!("Max weight: {}", graph.raw_edges().iter().map(|ref w| w.weight).max().expect("No weights in the graph!"));
    println!("Avg weight: {:.2}", graph.raw_edges().iter().map(|ref w| w.weight).fold(0usize, |s, w| s + w as usize) as f64 / graph.edge_count() as f64);
    // TODO
    // println!("Max in: {}", graph.values().fold(0usize, |mx, val| max(mx, val.in_num)));
    // println!("Max out: {}", graph.edges_directed(EdgeDirection::Outgoing).max_by(|&e| e.weights()).expect("No weights in the graph!"));
    // println!("Avg outgoing: {:.2}", (graph.edges_directed(EdgeDirection::Outgoing).fold(0usize, |m, &e| m + e.weights() as usize)) as f64 / graph.edge_count() as f64);
    let real_in = graph.externals(EdgeDirection::Incoming).count();
    let real_out = graph.externals(EdgeDirection::Outgoing).count();
    println!("Real in: {} ({:.2}%)", real_in, (real_in*100) as f64 / graph.node_count() as f64);
    println!("Real out: {} ({:.2}%)", real_out, (real_out*100) as f64 / graph.node_count() as f64);
    println!("{}", repeat("*").take(20).collect::<String>());
}
