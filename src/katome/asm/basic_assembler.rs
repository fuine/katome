//! Basic genome assembler.

use asm::{Assemble, Contigs, SEQUENCES};
use collections::{GIR, Graph, Convert};
use config::Config;
use prelude::{EdgeWeight, set_global_k_sizes};
use stats::Stats;

use std::path::Path;

/// Basic assembler.
pub struct BasicAsm {}

impl Assemble for BasicAsm {
    fn assemble<P: AsRef<Path>, G: Graph>(config: Config<P>) {
        info!("Starting assembler!");
        unsafe {
            set_global_k_sizes(config.k_mer_size);
        }
        let (graph, number_of_read_bytes) = G::create(config.input_path, config.input_file_type);
        let saved: usize = SEQUENCES.read().iter().map(|x| x.len()).sum();
        let total: usize = SEQUENCES.read().len();
        println!("Avg size of edge: {}", saved as f64 / total as f64);
        println!("I saved {} out of {} bytes -- {:.2}%",
                 saved,
                 number_of_read_bytes,
                 (saved * 100) as f64 / number_of_read_bytes as f64);
        assemble_with_graph(graph,
                            config.output_path,
                            config.original_genome_length,
                            config.minimal_weight_threshold);
    }

    fn assemble_with_gir<P: AsRef<Path>, G, T: GIR>(config: Config<P>) where G: Graph + Convert<T> {
        info!("Starting assembler!");
        unsafe {
            set_global_k_sizes(config.k_mer_size);
        }
        let (gir, number_of_read_bytes) = T::create(config.input_path, config.input_file_type);
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
        assemble_with_graph(graph,
                            config.output_path,
                            config.original_genome_length,
                            config.minimal_weight_threshold);
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
    let serialized_contigs = graph.collapse();
    println!("I created {} contigs", serialized_contigs.len());
    let contigs = Contigs::new(original_genome_length, serialized_contigs);
    info!("Stats for the contigs: {}", contigs.stats());
    contigs.print_stats();
    info!("All done!");
}
