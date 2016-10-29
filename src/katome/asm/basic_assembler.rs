//! Basic genome assembler.
use algorithms::builder::{Build, InputFileType};
// use algorithms::collapser::Collapsable;
use algorithms::pruner::Prunable;
use algorithms::standardizer::Standardizable;
use asm::{Assemble, SEQUENCES};
use data::collections::girs::Convert;
use data::collections::girs::hs_gir::HsGIR;
use data::collections::girs::hm_gir::HmGIR;
use data::collections::graphs::pt_graph::PtGraph;
use data::primitives::EdgeWeight;
use data::statistics::HasStats;
use std::path::Path;

/// Basic assembler.
pub struct BasicAsm {}

impl Assemble for BasicAsm {
    fn assemble<P: AsRef<Path>>(input: P, _output: P, original_genome_length: usize,
                                minimal_weight_threshold: usize, ft: InputFileType) {
        info!("Starting assembler!");
        // let (gir, number_of_read_bytes) = HmGIR::create(input, ft);
        let (mut graph, number_of_read_bytes) = PtGraph::create(input, ft);
        let saved: usize = SEQUENCES.read().iter().map(|x| x.len()).sum();
        let total: usize = SEQUENCES.read().len();
        println!("Avg size of edge: {}", saved as f64 / total as f64);
        println!("I saved {} out of {} bytes -- {:.2}%",
                 saved,
                 number_of_read_bytes,
                 (saved * 100) as f64 / number_of_read_bytes as f64);
        // gir.print_stats();
        // gir.remove_weak_edges(minimal_weight_threshold as EdgeWeight);
        // let mut graph = PtGraph::create_from(gir);
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
        // let contigs = graph.collapse();
        // println!("I created {} contigs", contigs.len());
        info!("All done!");
    }
}
