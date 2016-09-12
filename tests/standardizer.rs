#![feature(plugin)]
#![plugin(stainless)]
#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
extern crate katome;
extern crate petgraph;

pub use katome::algorithms::builder::Build;
pub use katome::algorithms::standardizer::Standardizable;
pub use katome::asm::assembler::lock::LOCK;
pub use katome::asm::assembler::SEQUENCES;
pub use katome::data::collections::graphs::pt_graph::{PtGraph, NodeIndex, EdgeIndex};
pub use katome::data::primitives::K_SIZE;
pub use katome::data::statistics::{Counts, HasStats, Opt, Stats};
pub use std::sync::Mutex;

describe! tests {
    before_each {
        // get global lock over sequences for testing
        let _l = LOCK.lock().unwrap();
        // Clear up SEQUENCES
        SEQUENCES.write().unwrap().clear();
        // hardcoded K_SIZE value for now :/
        assert_eq!(K_SIZE, 40);
    }

    describe! data1 {
        before_each {
            let (mut graph, _) = PtGraph::create("./tests/test_files/data1.txt".to_string());
            let correct_stats = vec![
                Stats {
                    capacity: (32, Opt::Full(32)),
                    counts: Counts {
                        node_count: 26,
                        edge_count: 26
                    },
                    max_edge_weight: Opt::Full(5),
                    avg_edge_weight: Opt::Full(4.77),
                    max_in_degree: Opt::Full(2),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(1.0),
                    incoming_vert_count: Opt::Full(1),
                    outgoing_vert_count: Opt::Full(0)
                },
                Stats {
                    capacity: (32, Opt::Full(32)),
                    counts: Counts {
                        node_count: 26,
                        edge_count: 26
                    },
                    max_edge_weight: Opt::Full(20),
                    avg_edge_weight: Opt::Full(1.73),
                    max_in_degree: Opt::Full(2),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(1.0),
                    incoming_vert_count: Opt::Full(1),
                    outgoing_vert_count: Opt::Full(0)
                }];
        }

        it "standardizes contigs" {
            // change the last edge in order to get observable results in standardize
            graph.add_edge(NodeIndex::new(25), NodeIndex::new(2), 70);
            graph.remove_edge(EdgeIndex::new(25));
            graph.standardize_contigs();
            assert_eq!(correct_stats[0], graph.stats());
        }

        it "standardizes edges" {
            graph.standardize_edges(60, 3);
            assert_eq!(correct_stats[1], graph.stats());
        }
    }

    describe! data2 {
        before_each {
            let (mut graph, _) = PtGraph::create("./tests/test_files/data2.txt".to_string());
            let correct_stats = vec![
                Stats {
                    capacity: (1024, Opt::Full(1024)),
                    counts: Counts {
                        node_count: 650,
                        edge_count: 650,
                    },
                    max_edge_weight: Opt::Full(831),
                    avg_edge_weight: Opt::Full(2.35),
                    max_in_degree: Opt::Full(5),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(1.0),
                    incoming_vert_count: Opt::Full(25),
                    outgoing_vert_count: Opt::Full(0),
                },
                Stats {
                    capacity: (1024, Opt::Full(1024)),
                    counts: Counts {
                        node_count: 9,
                        edge_count: 9
                    },
                    max_edge_weight: Opt::Full(24),
                    avg_edge_weight: Opt::Full(3.56),
                    max_in_degree: Opt::Full(4),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(1.0),
                    incoming_vert_count: Opt::Full(5),
                    outgoing_vert_count: Opt::Full(0)
                }];
        }

        it "standardizes contigs" {
            graph.standardize_contigs();
            assert_eq!(correct_stats[0], graph.stats());
        }

        it "standardizes edges" {
            graph.standardize_edges(65, 3);
            assert_eq!(correct_stats[1], graph.stats());
        }
    }

    // It's no use standardizing data3 as all edges have weight of 1.
}
