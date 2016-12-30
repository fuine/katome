//! Basic genome assembler.

use asm::{Assemble, Contigs, SEQUENCES};
use collections::{GIR, Graph, Convert};
use config::Config;
use prelude::{EdgeWeight, K_SIZE, set_global_k_sizes};
use stats::Stats;

use std::path::Path;
use std::time::Instant;

/// Basic assembler.
pub struct BasicAsm {}

impl Assemble for BasicAsm {
    fn assemble<P: AsRef<Path>, G: Graph>(config: Config<P>) {
        let start = Instant::now();
        info!("Starting assembler!");
        unsafe {
            set_global_k_sizes(config.k_mer_size);
        }
        let (graph, number_of_read_bytes) =
            G::create(config.input_path,
                      config.input_file_type,
                      config.reverse_complement,
                      config.minimal_weight_threshold as EdgeWeight);
        sequences_stats(number_of_read_bytes);
        assemble_with_graph(graph,
                            config.output_path,
                            config.original_genome_length,
                            config.minimal_weight_threshold,
                            start);
    }

    fn assemble_with_gir<P: AsRef<Path>, G, T: GIR>(config: Config<P>) where G: Graph + Convert<T> {
        let start = Instant::now();
        info!("Starting assembler!");
        unsafe {
            set_global_k_sizes(config.k_mer_size);
        }
        let (gir, number_of_read_bytes) = T::create(config.input_path,
                                                    config.input_file_type,
                                                    config.reverse_complement,
                                                    config.minimal_weight_threshold as EdgeWeight);
        sequences_stats(number_of_read_bytes);
        gir.log_stats();
        let graph = G::create_from(gir);
        assemble_with_graph(graph,
                            config.output_path,
                            config.original_genome_length,
                            config.minimal_weight_threshold,
                            start);
    }
}

fn sequences_stats(number_of_read_bytes: usize) {
    let saved: usize = SEQUENCES.read().iter().map(|x| x.len()).sum();
    let total: usize = SEQUENCES.read().len();
    info!("Avg size of edge: {}", saved as f64 / total as f64);
    info!("Saved {} out of {} bytes bytes -- {:.2}%",
             saved,
             number_of_read_bytes,
             (saved * 100) as f64 / number_of_read_bytes as f64);
}

fn assemble_with_graph<P: AsRef<Path>, G: Graph>(mut graph: G, _output: P,
                                                 original_genome_length: usize,
                                                 minimal_weight_threshold: usize, start: Instant) {
    graph.log_stats();
    info!("First pruning.");
    graph.remove_dead_paths();
    graph.log_stats();
    info!("Standardizing contigs.");
    graph.standardize_contigs();
    graph.remove_weak_edges(minimal_weight_threshold as EdgeWeight);
    graph.standardize_contigs();
    graph.log_stats();
    info!("Standardizing edges");
    graph.standardize_edges(original_genome_length,
                            unsafe { K_SIZE },
                            minimal_weight_threshold as EdgeWeight);
    graph.log_stats();
    info!("Second pruning");
    graph.remove_dead_paths();
    graph.log_stats();
    let serialized_contigs = graph.collapse();
    info!("I created {} contigs", serialized_contigs.len());
    let contigs = Contigs::new(original_genome_length, serialized_contigs);
    contigs.log_stats();
    contigs.save_to_file(_output);
    let duration = start.elapsed();
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = secs / 60 - hours * 60;
    let seconds = secs % 60;
    info!("All done! Total elapsed time: {:02}h {:02}m {:02}.{}s", hours, minutes, seconds, duration.subsec_nanos());
}
