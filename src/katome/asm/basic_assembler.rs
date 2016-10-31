//! Basic genome assembler.

use algorithms::builder::InputFileType;
use asm::{Assemble, SEQUENCES};
use data::collections::girs::{GIR, Convert};
use data::collections::graphs::Graph;
use data::contigs_statistics::HasContigsStats;
use data::primitives::EdgeWeight;

use std::path::Path;

/// Basic assembler.
pub struct BasicAsm {}

impl Assemble for BasicAsm {
    fn assemble<P: AsRef<Path>, G: Graph>(input: P, _output: P, original_genome_length: usize,
                                          minimal_weight_threshold: usize, ft: InputFileType) {
        info!("Starting assembler!");
        let (graph, number_of_read_bytes) = G::create(input, ft);
        let saved: usize = SEQUENCES.read().iter().map(|x| x.len()).sum();
        let total: usize = SEQUENCES.read().len();
        println!("Avg size of edge: {}", saved as f64 / total as f64);
        println!("I saved {} out of {} bytes -- {:.2}%",
                 saved,
                 number_of_read_bytes,
                 (saved * 100) as f64 / number_of_read_bytes as f64);
        assemble_with_graph(graph, _output, original_genome_length, minimal_weight_threshold);
    }

    fn assemble_with_gir<P: AsRef<Path>, G: Graph, T: GIR>(input: P, _output: P,
                                                           original_genome_length: usize,
                                                           minimal_weight_threshold: usize,
                                                           ft: InputFileType)
        where G: Graph + Convert<T> {

        info!("Starting assembler!");
        let (gir, number_of_read_bytes) = T::create(input, ft);
        let saved: usize = SEQUENCES.read().iter().map(|x| x.len()).sum();
        let total: usize = SEQUENCES.read().len();
        println!("Avg size of edge: {}", saved as f64 / total as f64);
        println!("I saved {} out of {} bytes -- {:.2}%",
                 saved,
                 number_of_read_bytes,
                 (saved * 100) as f64 / number_of_read_bytes as f64);
        gir.print_stats();
        // gir.remove_weak_edges(minimal_weight_threshold as EdgeWeight);
        let graph = G::create_from(gir);
        assemble_with_graph(graph, _output, original_genome_length, minimal_weight_threshold);
    }
}

fn assemble_with_graph<P: AsRef<Path>, G: Graph>(mut graph: G, _output: P,
                                                 original_genome_length: usize,
                                                 minimal_weight_threshold: usize) {
    graph.print_stats();
    println!("First pruning.");
    graph.remove_dead_paths();
    graph.print_stats();
    println!("Standardizing contigs.");
    graph.standardize_contigs();
    graph.print_stats();
    println!("Standardizing edges");
    graph.standardize_edges(original_genome_length, minimal_weight_threshold as EdgeWeight);
    graph.print_stats();
    println!("Second pruning");
    graph.remove_dead_paths();
    graph.print_stats();
    println!("Collapsing!");
    let contigs = graph.collapse();
    println!("I created {} contigs", contigs.len());
    contigs.print_stats(original_genome_length);
    info!("All done!");
}
