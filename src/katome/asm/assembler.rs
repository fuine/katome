use ::data::input::{read_sequences};
use ::data::types::{Graph, VecArc, WEAK_EDGE_THRESHOLD, VertexId};
use ::algorithms::pruner::{Pruner};
use ::algorithms::hardener::remove_weak_edges;
use ::algorithms::standarizer::standarize_edges;
use ::algorithms::collapser::collapse;
use std::sync::Arc;
use std::iter::repeat;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::BTreeSet;

pub fn assemble(input: String, output: String, original_genome_length: usize) {
    info!("Starting assembler!");
    warn!("test");
    error!("test2");
    let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
    // let mut graph: Graph = Graph::with_capacity_and_hasher(91008059, MyHasher::default());
    let mut graph: Graph = Graph::default();
    let mut saved_counter = 0;
    let mut number_of_reads: usize = 0;
    let mut number_of_read_bytes = 0;
    read_sequences(input, sequences.clone(), &mut graph,
                   &mut saved_counter, &mut number_of_read_bytes,
                   &mut number_of_reads);
    print_stats_with_savings(&graph, saved_counter, number_of_read_bytes);
    sequences.borrow_mut().shrink_to_fit();
    remove_weak_edges(&mut graph, sequences.clone(), WEAK_EDGE_THRESHOLD);
    print_stats_with_savings(&graph, saved_counter, number_of_read_bytes);
    {
        Pruner::new(&mut graph, sequences.clone()).prune_graph();
    }
    print_stats_with_savings(&graph, saved_counter, number_of_read_bytes);
    graph.shrink_to_fit();
    let average_read_length: f64 = number_of_read_bytes as f64 / number_of_reads as f64;
    println!("Standarizing!");
    standarize_edges(&mut graph, sequences.clone(), original_genome_length, number_of_reads, average_read_length);
    print_stats_with_savings(&graph, saved_counter, number_of_read_bytes);
    println!("Collapsing!");
    collapse(&mut graph, sequences.clone(), output);
    info!("All done!");
}

#[allow(dead_code)]
fn print_graph_representation(graph: &Graph) {
    for (key, val) in graph.iter() {
        println!("{}: {} {:?}", key.name(), val.in_num, val.outgoing.iter().fold(Vec::new() as Vec<u16>, |mut vec, ref x| {vec.push(x.1); vec}));
    }
}
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

pub fn print_stats_with_savings(graph: &Graph, saved_counter: usize, number_of_read_bytes: usize) {
    println!("I saved {} out of {} bytes -- {:.2}%", saved_counter, number_of_read_bytes, (saved_counter*100) as f64/number_of_read_bytes as f64);
    print_stats(graph);
}

pub fn print_stats(graph: &Graph) {
    println!("I have the capacity of {} for {} stored sequences", graph.capacity(), graph.len());
    println!("Max weight: {}", graph.values().fold(0u16, |mx, val| max(mx, val.outgoing.iter().fold(0u16, |m, v| max(m, v.1)))));
    println!("Avg weight: {}", graph.values().fold(0usize, |mx, val| mx + val.outgoing.iter().fold(0u16, |m, v| m + v.1) as usize) as f64 / graph.len() as f64);
    println!("Max in: {}", graph.values().fold(0usize, |mx, val| max(mx, val.in_num)));
    println!("Max out: {}", graph.values().fold(0usize, |mx, val| max(mx, val.outgoing.len())));
    println!("Avg outgoing: {:.2}", (graph.values().fold(0usize, |mx, val| mx + val.outgoing.len())) as f64 / graph.len() as f64);
    let real_in = graph.values().filter(|&val| val.in_num == 0).count();
    let real_out = graph.values().filter(|&val| val.outgoing.len() == 0).count();
    println!("Real in: {} ({:.2}%)", real_in, (real_in*100) as f64 / graph.len() as f64);
    println!("Real out: {} ({:.2}%)", real_out, (real_out*100) as f64 / graph.len() as f64);
    println!("{}", repeat("*").take(20).collect::<String>());
}
