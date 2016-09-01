use ::data::graph::{EdgeWeight, VecArc};
use ::data::gir::{create_gir, gir_to_graph};
use ::data::statistics::HasStats;
use ::algorithms::pruner::Prunable;
use ::algorithms::standardizer::Standardizable;
use ::algorithms::collapser::get_contigs;
use std::sync::{Arc, RwLock};


lazy_static! {
    /// Global mutable vector of bytes. Contains unique sequences, most of which
    /// have common parts amongst themselves.
    ///
    /// `ReadSlice` uses offsets on this structure to efficiently store
    /// information about sequence. Global container allows to save 8 bytes in
    /// ReadSlice (it doesn't have to store `Arc` to the container).
    pub static ref SEQUENCES: VecArc = Arc::new(RwLock::new(Vec::new()));
}

pub mod lock {
    use std::sync::Mutex;
    // mutex over sequences specifically for tests
    lazy_static! {
        pub static ref LOCK: Mutex<usize> = Mutex::new(0);
    }
}

pub fn assemble(input: String, output: String, original_genome_length: usize,
                minimal_weight_threshold: usize) {
    info!("Starting assembler!");
    let (gir, number_of_read_bytes) = create_gir(input);
    println!("I saved {} out of {} bytes -- {:.2}%",
             unwrap!(SEQUENCES.read()).len(),
             number_of_read_bytes,
             (unwrap!(SEQUENCES.read()).len() * 100) as f64 / number_of_read_bytes as f64);
    gir.print_stats();
    let mut graph = gir_to_graph(gir);
    graph.print_stats();
    println!("First pruning.");
    graph.remove_dead_paths();
    graph.print_stats();
    println!("Standardizing contigs.");
    graph.standardize_contigs();
    graph.print_stats();
    println!("Standardizing edges");
    graph.standardize_edges(original_genome_length,
                            minimal_weight_threshold as EdgeWeight);
    graph.print_stats();
    println!("Second pruning");
    graph.remove_dead_paths();
    graph.print_stats();
    println!("Collapsing!");
    let contigs = get_contigs(graph);
    println!("I created {} contigs", contigs.len());
    info!("All done!");
}
