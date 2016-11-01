#![feature(plugin)]
#![plugin(stainless)]
#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
extern crate katome;
extern crate petgraph;

pub use katome::config::InputFileType;
pub use katome::algorithms::builder::Build;
pub use katome::algorithms::pruner::{Clean, Prunable};
pub use katome::asm::SEQUENCES;
pub use katome::asm::lock::LOCK;
pub use katome::collections::{Convert, HmGIR, PtGraph};
pub use katome::prelude::K_SIZE;
pub use katome::stats::{Counts, Opt, CollectionStats, Stats};
pub use std::sync::Mutex;
pub use std::f64;

describe! pruner {
    before_each {
        // get global lock over sequences for testing
        let _l = LOCK.lock().unwrap();
        // Clear up SEQUENCES
        {
            let mut s = SEQUENCES.write();
            s.clear();
            s.push(vec![].into_boxed_slice());
        }
        // hardcoded K_SIZE value for now :/
        assert_eq!(K_SIZE, 40);
    }

    describe! data1 {
        before_each {
            let correct_stats = vec![
                CollectionStats {
                    capacity: (64, Opt::Full(64)),
                    counts: Counts {
                        node_count: 62,
                        edge_count: 61,
                    },
                    max_edge_weight: Opt::Full(2),
                    avg_edge_weight: Opt::Full(2.0),
                    max_in_degree: Opt::Full(1),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(0.98),
                    incoming_vert_count: Opt::Full(1),
                    outgoing_vert_count: Opt::Full(1)
                },
                CollectionStats {
                    capacity: (64, Opt::Full(64)),
                    counts: Counts {
                        node_count: 0,
                        edge_count: 0
                    },
                    max_edge_weight: Opt::Full(0),
                    avg_edge_weight: Opt::Full(f64::NAN),
                    max_in_degree: Opt::Full(0),
                    max_out_degree: Opt::Full(0),
                    avg_out_degree: Opt::Full(f64::NAN),
                    incoming_vert_count: Opt::Full(0),
                    outgoing_vert_count: Opt::Full(0)
                },
                ];
        }

        describe! graph {
            before_each {
                let (mut graph, _) = PtGraph::create("./tests/test_files/data1.txt".to_string(), InputFileType::Fastq);
            }

            it "removes single vertices" {
                // TODO remove some edges to show that vertices will get removed?
                graph.remove_single_vertices();
                assert_eq!(correct_stats[0], graph.stats());
            }

            it "removes weak edges" {
                graph.remove_weak_edges(3);
                assert_eq!(correct_stats[1].counts, graph.stats().counts);
            }

            it "removes dead paths" {
                graph.remove_dead_paths();
                assert_eq!(correct_stats[1].counts, graph.stats().counts);
            }
        }

        describe! gir {
            before_each {
                let (mut gir, _) = HmGIR::create("./tests/test_files/data1.txt".to_string(), InputFileType::Fastq);
            }

            it "removes single vertices" {
                gir.remove_single_vertices();
                assert_eq!(correct_stats[0].counts, gir.stats().counts);
                let graph = PtGraph::create_from(gir);
                assert_eq!(correct_stats[0], graph.stats());
            }

            it "removes weak edges" {
                gir.remove_weak_edges(3);
                assert_eq!(correct_stats[1].counts, gir.stats().counts);
            }
        }

    }

    describe! data2 {
        before_each {
            let correct_stats = vec![
                CollectionStats {
                    capacity: (8192, Opt::Full(8192)),
                    counts: Counts {
                        node_count: 5704,
                        edge_count: 5612,
                    },
                    max_edge_weight: Opt::Full(1),
                    avg_edge_weight: Opt::Full(1.0),
                    max_in_degree: Opt::Full(1),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(0.98),
                    incoming_vert_count: Opt::Full(92),
                    outgoing_vert_count: Opt::Full(92)
                },
                CollectionStats {
                    capacity: (64, Opt::Full(64)),
                    counts: Counts {
                        node_count: 0,
                        edge_count: 0
                    },
                    max_edge_weight: Opt::Full(0),
                    avg_edge_weight: Opt::Full(f64::NAN),
                    max_in_degree: Opt::Full(0),
                    max_out_degree: Opt::Full(0),
                    avg_out_degree: Opt::Full(f64::NAN),
                    incoming_vert_count: Opt::Full(0),
                    outgoing_vert_count: Opt::Full(0)
                }];
        }
        describe! graph {
            before_each {
                let (mut graph, _) = PtGraph::create("./tests/test_files/data2.txt".to_string(), InputFileType::Fastq);
            }

            it "removes single vertices" {
                graph.remove_single_vertices();
                assert_eq!(correct_stats[0], graph.stats());
            }

            it "removes weak edges" {
                graph.remove_weak_edges(2);
                assert_eq!(correct_stats[1].counts, graph.stats().counts);
            }

            it "removes dead paths" {
                graph.remove_dead_paths();
                assert_eq!(correct_stats[1].counts, graph.stats().counts);
            }
        }

        /* describe! gir {
            before_each {
                let (mut gir, _) = HmGIR::create("./tests/test_files/data2.txt".to_string(), InputFileType::Fastq);
            }

            it "removes single vertices" {
                gir.remove_single_vertices();
                assert_eq!(correct_stats[0].counts, gir.stats().counts);
                let graph = PtGraph::create_from(gir);
                assert_eq!(correct_stats[0], graph.stats());
            }

            it "removes weak edges" {
                gir.remove_weak_edges(2);
                assert_eq!(correct_stats[1].counts, gir.stats().counts);
            }
        } */

    }

    describe! data3 {
        // TODO change something in order to show better weak edges removal
        before_each {
            let correct_stats = vec![
                CollectionStats {
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
                CollectionStats {
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
                CollectionStats {
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

        describe! graph {
            before_each {
                let (mut graph, _) = PtGraph::create("./tests/test_files/data3.txt".to_string(), InputFileType::Fastq);
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

        describe! gir {
            before_each {
                let (mut gir, _) = HmGIR::create("./tests/test_files/data3.txt".to_string(), InputFileType::Fastq);
            }

            it "removes single vertices" {
                gir.remove_single_vertices();
                assert_eq!(correct_stats[0].counts, gir.stats().counts);
                let graph = PtGraph::create_from(gir);
                assert_eq!(correct_stats[0], graph.stats());
            }

            it "removes weak edges" {
                gir.remove_weak_edges(1);
                assert_eq!(correct_stats[1].counts, gir.stats().counts);
                let graph = PtGraph::create_from(gir);
                assert_eq!(correct_stats[1], graph.stats());
            }
        }
    }
}
