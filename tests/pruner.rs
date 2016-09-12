#![feature(plugin)]
#![plugin(stainless)]
#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
extern crate katome;
extern crate petgraph;

pub use katome::algorithms::builder::Build;
pub use katome::algorithms::pruner::{Clean, Prunable};
pub use katome::asm::assembler::lock::LOCK;
pub use katome::asm::assembler::SEQUENCES;
pub use katome::data::collections::graphs::pt_graph::PtGraph;
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
            let (mut graph, _) = PtGraph::create("./tests/test_files/data1.txt".to_string(), false);
            let correct_stats = vec![
                Stats {
                    capacity: (32, Opt::Full(32)),
                    counts: Counts {
                        node_count: 26,
                        edge_count: 26
                    },
                    max_edge_weight: Opt::Full(72),
                    avg_edge_weight: Opt::Full(4.69),
                    max_in_degree: Opt::Full(2),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(1.0),
                    incoming_vert_count: Opt::Full(1),
                    outgoing_vert_count: Opt::Full(0)
                },
                Stats {
                    capacity: (32, Opt::Full(32)),
                    counts: Counts {
                        node_count: 1,
                        edge_count: 1
                    },
                    max_edge_weight: Opt::Full(72),
                    avg_edge_weight: Opt::Full(72.0),
                    max_in_degree: Opt::Full(1),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(1.0),
                    incoming_vert_count: Opt::Full(0),
                    outgoing_vert_count: Opt::Full(0)
                },
                Stats {
                    capacity: (32, Opt::Full(32)),
                    counts: Counts {
                        node_count: 26,
                        edge_count: 26
                    },
                    max_edge_weight: Opt::Full(72),
                    avg_edge_weight: Opt::Full(4.69),
                    max_in_degree: Opt::Full(2),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(1.0),
                    incoming_vert_count: Opt::Full(1),
                    outgoing_vert_count: Opt::Full(0)
                }];
        }

        it "removes single vertices" {
            // TODO remove some edges to show that vertices will get removed?
            graph.remove_single_vertices();
            assert_eq!(correct_stats[0], graph.stats());
        }

        it "removes weak edges" {
            graph.remove_weak_edges(3);
            assert_eq!(correct_stats[1], graph.stats());
        }

        it "removes dead paths" {
            graph.remove_dead_paths();
            assert_eq!(correct_stats[2], graph.stats());
        }
    }

    describe! data2 {
        before_each {
            let (mut graph, _) = PtGraph::create("./tests/test_files/data2.txt".to_string(), false);
            let correct_stats = vec![
                Stats {
                    capacity: (1024, Opt::Full(1024)),
                    counts: Counts {
                        node_count: 650,
                        edge_count: 650
                    },
                    max_edge_weight: Opt::Full(831),
                    avg_edge_weight: Opt::Full(2.35),
                    max_in_degree: Opt::Full(5),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(1.0),
                    incoming_vert_count: Opt::Full(25),
                    outgoing_vert_count: Opt::Full(0)
                },
                Stats {
                    capacity: (1024, Opt::Full(1024)),
                    counts: Counts {
                        node_count: 23,
                        edge_count: 23
                    },
                    max_edge_weight: Opt::Full(831),
                    avg_edge_weight: Opt::Full(39.04),
                    max_in_degree: Opt::Full(5),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(1.0),
                    incoming_vert_count: Opt::Full(9),
                    outgoing_vert_count: Opt::Full(0)
                },
                Stats {
                    capacity: (1024, Opt::Full(1024)),
                    counts: Counts {
                        node_count: 1,
                        edge_count: 1
                    },
                    max_edge_weight: Opt::Full(831),
                    avg_edge_weight: Opt::Full(831.0),
                    max_in_degree: Opt::Full(1),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(1.0),
                    incoming_vert_count: Opt::Full(0),
                    outgoing_vert_count: Opt::Full(0)
                }];
        }

        it "removes single vertices" {
            graph.remove_single_vertices();
            assert_eq!(correct_stats[0], graph.stats());
        }

        it "removes weak edges" {
            graph.remove_weak_edges(2);
            assert_eq!(correct_stats[1], graph.stats());
        }

        it "removes dead paths" {
            graph.remove_dead_paths();
            assert_eq!(correct_stats[2], graph.stats());
        }
    }

    describe! data3 {
        // TODO change something in order to show better weak edges removal
        before_each {
            let (mut graph, _) = PtGraph::create("./tests/test_files/data3.txt".to_string(), false);
            let correct_stats = vec![
                Stats {
                    capacity: (16384, Opt::Full(16384)),
                    counts: Counts {
                        node_count: 14446,
                        edge_count: 14213
                    },
                    max_edge_weight: Opt::Full(1),
                    avg_edge_weight: Opt::Full(1.0),
                    max_in_degree: Opt::Full(1),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(0.98),
                    incoming_vert_count: Opt::Full(233),
                    outgoing_vert_count: Opt::Full(233)
                },
                Stats {
                    capacity: (16384, Opt::Full(16384)),
                    counts: Counts {
                        node_count: 14446,
                        edge_count: 14213
                    },
                    max_edge_weight: Opt::Full(1),
                    avg_edge_weight: Opt::Full(1.0),
                    max_in_degree: Opt::Full(1),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(0.98),
                    incoming_vert_count: Opt::Full(233),
                    outgoing_vert_count: Opt::Full(233)
                },
                Stats {
                    capacity: (16384, Opt::Full(16384)),
                    counts: Counts {
                        node_count: 0,
                        edge_count: 0
                    },
                    max_edge_weight: Opt::Full(0),
                    avg_edge_weight: Opt::Full(0.0),
                    max_in_degree: Opt::Full(0),
                    max_out_degree: Opt::Full(0),
                    avg_out_degree: Opt::Full(0.0),
                    incoming_vert_count: Opt::Full(0),
                    outgoing_vert_count: Opt::Full(0)
                },

            ];
        }

        it "removes single vertices" {
            graph.remove_single_vertices();
            assert_eq!(correct_stats[0], graph.stats());
        }

        it "removes weak edges" {
            graph.remove_weak_edges(1);
            assert_eq!(correct_stats[1], graph.stats());
        }

        it "removes dead paths" {
            graph.remove_dead_paths();
            assert_eq!(correct_stats[2].counts, graph.stats().counts);
        }
    }
}
