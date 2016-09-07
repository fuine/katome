//! Basic genome assembler.
use data::primitives::{EdgeWeight, VecArc};
use data::collections::girs::gir::Convert;
use data::collections::girs::hs_gir::HsGIR;
use data::collections::graphs::pt_graph::PtGraph;
use algorithms::builder::Build;
use ::data::statistics::HasStats;
use ::algorithms::pruner::Prunable;
use ::algorithms::standardizer::Standardizable;
use algorithms::collapser::Collapsable;


lazy_static! {
    /// Global mutable vector of bytes. Contains unique reads slices (k-mers).
    ///
    /// `ReadSlice` uses offsets on this structure to efficiently store
    /// information about sequence. Global container allows to save 8 bytes in
    /// ReadSlice (it doesn't have to store `Arc` to the container).
    pub static ref SEQUENCES: VecArc = VecArc::default();
}

#[doc(hidden)]
pub mod lock {
    use std::sync::Mutex;
    // mutex over sequences specifically for tests
    lazy_static! {
        pub static ref LOCK: Mutex<()> = Mutex::new(());
    }
}

/// Assembles given data and writes results into the output file.
///
/// * `input` - Fastaq or fasta input file.
/// * `output` - Path to the output file. Each line in the output file denotes single contig.
/// * `original_genome_length` - length of the reference genome.
/// * `minimal_weight_threshold` - threshold used by `Pruner`.
pub fn assemble(input: String, _output: String, original_genome_length: usize,
                minimal_weight_threshold: usize) {
    info!("Starting assembler!");
    let (gir, number_of_read_bytes) = HsGIR::create(input, false);
    println!("I saved {} out of {} bytes -- {:.2}%",
             unwrap!(SEQUENCES.read()).len(),
             number_of_read_bytes,
             (unwrap!(SEQUENCES.read()).len() * 100) as f64 / number_of_read_bytes as f64);
    gir.print_stats();
    let mut graph = PtGraph::create_from(gir);
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
    let contigs = graph.collapse();
    println!("I created {} contigs", contigs.len());
    info!("All done!");
}
