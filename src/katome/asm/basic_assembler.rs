//! Basic genome assembler.
use algorithms::builder::Build;
use algorithms::collapser::Collapsable;
use algorithms::pruner::Prunable;
use algorithms::standardizer::Standardizable;
use asm::{Assemble, SEQUENCES};
use data::collections::girs::Convert;
use data::collections::girs::hs_gir::HsGIR;
use data::collections::graphs::pt_graph::PtGraph;
use data::primitives::EdgeWeight;
use data::statistics::HasStats;
use std::path::Path;

/// Basic assembler.
pub struct BasicAsm {}

impl Assemble for BasicAsm {
    fn assemble<P: AsRef<Path>>(input: P, _output: P, original_genome_length: usize,
                minimal_weight_threshold: usize) {
        info!("Starting assembler!");
        let (gir, number_of_read_bytes) = HsGIR::create(input);
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
}
