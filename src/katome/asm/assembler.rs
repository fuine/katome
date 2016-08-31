use ::data::graph::{Graph, VecArc, EdgeWeight};
use ::data::gir::{GIR, create_gir, gir_to_graph};
use ::algorithms::pruner::{remove_dead_paths};
use ::algorithms::standardizer::{standardize_edges, standardize_contigs};
use ::algorithms::collapser::get_contigs;
use std::sync::{Arc, RwLock};
use std::iter::repeat;
use ::petgraph::EdgeDirection;


lazy_static! {
    /// Global mutable vector of bytes. Contains unique sequences, most of which
    /// have common parts amongst themselves.
    ///
    /// `ReadSlice` uses offsets on this structure to efficiently store
    /// information about sequence. Global container allows to save 8 bytes in
    /// ReadSlice (it doesn't have to store `Arc` to the container).
    pub static ref SEQUENCES: VecArc = Arc::new(RwLock::new(Vec::new()));
}

pub fn assemble(input: String, output: String, original_genome_length: usize, minimal_weight_threshold: usize) {
    info!("Starting assembler!");
    let (gir, number_of_read_bytes) = create_gir(input);
    print_stats_for_gir(&gir, number_of_read_bytes);
    let mut graph = gir_to_graph(gir);
    print_stats_with_savings(&graph, number_of_read_bytes);
    println!("First pruning.");
    remove_dead_paths(&mut graph);
    print_stats_with_savings(&graph, number_of_read_bytes);
    println!("Standardizing contigs.");
    standardize_contigs(&mut graph);
    print_stats_with_savings(&graph, number_of_read_bytes);
    // graph.shrink_to_fit();
    println!("Standardizing edges");
    standardize_edges(&mut graph, original_genome_length, minimal_weight_threshold as EdgeWeight);
    print_stats_with_savings(&graph, number_of_read_bytes);
    println!("Second pruning");
    remove_dead_paths(&mut graph);
    print_stats_with_savings(&graph, number_of_read_bytes);
    println!("Collapsing!");
    let contigs = get_contigs(graph);
    println!("I created {} contigs", contigs.len());
    // collapse(&mut graph, output);
    info!("All done!");
}

pub fn print_stats_for_gir(gir: &GIR, number_of_read_bytes: usize) {
    println!("I saved {} out of {} bytes -- {:.2}%", unwrap!(SEQUENCES.read()).len(), number_of_read_bytes, (unwrap!(SEQUENCES.read()).len()*100) as f64/number_of_read_bytes as f64);
    println!("I have the capacity of {:?} for {} nodes and {} edges", gir.capacity(), gir.len(), gir.iter().map(|ref e| e.edges.outgoing.len()).sum::<usize>());
}

pub fn print_stats_with_savings(graph: &Graph, number_of_read_bytes: usize) {
    println!("I saved {} out of {} bytes -- {:.2}%", unwrap!(SEQUENCES.read()).len(), number_of_read_bytes, (unwrap!(SEQUENCES.read()).len()*100) as f64/number_of_read_bytes as f64);
    print_stats(graph);
}

pub fn print_stats(graph: &Graph) {
    println!("I have the capacity of {:?} for {} nodes and {} edges", graph.capacity(), graph.node_count(), graph.edge_count());
    println!("Max weight: {}", unwrap!(graph.raw_edges().iter().map(|ref w| w.weight).max(), "No weights in the graph!"));
    println!("Avg weight: {:.2}", graph.raw_edges().iter().map(|ref w| w.weight).fold(0usize, |s, w| s + w as usize) as f64 / graph.edge_count() as f64);
    /* println!("Max in: {}", graph.node_indices().map(|n| in_degree(graph, n)).max().unwrap());
    println!("Max out: {}", graph.node_indices().map(|n| out_degree(graph, n)).max().expect("No nodes in the graph!"));
    println!("Avg outgoing: {:.2}", (graph.node_indices().fold(0usize, |m, n|{
        m + out_degree(graph, n)
        })) as f64 / graph.node_count() as f64); */
    let real_in = graph.externals(EdgeDirection::Incoming).count();
    let real_out = graph.externals(EdgeDirection::Outgoing).count();
    println!("Real in: {} ({:.2}%)", real_in, (real_in*100) as f64 / graph.node_count() as f64);
    println!("Real out: {} ({:.2}%)", real_out, (real_out*100) as f64 / graph.node_count() as f64);
    println!("{}", repeat("*").take(20).collect::<String>());
}
