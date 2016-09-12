#![feature(plugin)]
#![plugin(stainless)]
#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
extern crate katome;
pub use katome::algorithms::builder::Build;
pub use katome::data::collections::girs::hs_gir::HsGIR;
pub use katome::data::collections::girs::hm_gir::HmGIR;
pub use katome::data::collections::graphs::pt_graph::PtGraph;
pub use katome::data::primitives::K_SIZE;
pub use katome::asm::assembler::{SEQUENCES};
pub use katome::asm::assembler::lock::LOCK;
pub use katome::data::statistics::{Counts, HasStats, Stats, Opt};
pub use std::sync::Mutex;
pub use std::panic;

describe! build {
    before_each {
        // get global lock over sequences for testing
        let _l = LOCK.lock().unwrap();
        // Clear up SEQUENCES
        SEQUENCES.write().unwrap().clear();
        // hardcoded K_SIZE value for now :/
        assert_eq!(K_SIZE, 40);
        let _correct_read_bytes = vec![200, 2500, 23300];
        let _correct_counts = vec![(26, 26), (650, 650), (14446, 14213)];
    }
    describe! girs {
        before_each {
            let _correct_stats: Vec<Stats> = _correct_counts.iter().map(|&(x, y)| Stats::with_counts(x, y)).collect();
        }

        describe! HsGIR {
            before_each {
                pub use super::super::*;
            }

            it "builds data1" {
                let (gir, number_of_read_bytes) = HsGIR::create("./tests/test_files/data1.txt".to_string());
                assert_eq!(number_of_read_bytes, _correct_read_bytes[0]);
                assert_eq!(_correct_stats[0], gir.stats());
            }

            it "builds data2" {
                let (gir, number_of_read_bytes) = HsGIR::create("./tests/test_files/data2.txt".to_string());
                assert_eq!(number_of_read_bytes, _correct_read_bytes[1]);
                assert_eq!(_correct_stats[1], gir.stats());
            }

            it "builds data3" {
                let (gir, number_of_read_bytes) = HsGIR::create("./tests/test_files/data3.txt".to_string());
                assert_eq!(number_of_read_bytes, _correct_read_bytes[2]);
                assert_eq!(_correct_stats[2], gir.stats());
            }

            // use catch_unwind as to not to poison global SEQUENCE mutex
            it "fails for input with too short read" {
                let result = panic::catch_unwind(|| {
                    HsGIR::create("./tests/test_files/data_too_short_read.txt".to_string());
                });
                assert!(result.is_err());
            }

        }
        describe! HmGIR {
            before_each {
                pub use super::super::*;
            }

            it "builds data1" {
                let (gir, number_of_read_bytes) = HmGIR::create("./tests/test_files/data1.txt".to_string());
                assert_eq!(number_of_read_bytes, _correct_read_bytes[0]);
                assert_eq!(_correct_stats[0], gir.stats());
            }

            it "builds data2" {
                let (gir, number_of_read_bytes) = HmGIR::create("./tests/test_files/data2.txt".to_string());
                assert_eq!(number_of_read_bytes, _correct_read_bytes[1]);
                assert_eq!(_correct_stats[1], gir.stats());
            }

            it "builds data3" {
                let (gir, number_of_read_bytes) = HmGIR::create("./tests/test_files/data3.txt".to_string());
                assert_eq!(number_of_read_bytes, _correct_read_bytes[2]);
                assert_eq!(_correct_stats[2], gir.stats());
            }

            // use catch_unwind as to not to poison global SEQUENCE mutex
            it "fails for input with too short read" {
                let result = panic::catch_unwind(|| {
                    HsGIR::create("./tests/test_files/data_too_short_read.txt".to_string());
                });
                assert!(result.is_err());
            }
        }
    }
    describe! graphs {
        before_each {
            let _correct_stats = vec![
                Stats {
                    capacity: (32, Opt::Full(32)),
                    counts: Counts {
                        node_count: _correct_counts[0].0,
                        edge_count: _correct_counts[0].1,
                    },
                    max_edge_weight: Opt::Full(72),
                    avg_edge_weight: Opt::Full(4.69),
                    max_in_degree: Opt::Full(2),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(1.0),
                    incoming_vert_count: Opt::Full(1),
                    outgoing_vert_count: Opt::Full(0),
                },
                Stats {
                    capacity: (1024, Opt::Full(1024)),
                    counts: Counts {
                       node_count: _correct_counts[1].0,
                       edge_count: _correct_counts[1].1,
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
                    capacity: (16384, Opt::Full(16384)),
                    counts: Counts {
                        node_count: _correct_counts[2].0,
                        edge_count: _correct_counts[2].1,
                    },
                    max_edge_weight: Opt::Full(1),
                    avg_edge_weight: Opt::Full(1.0),
                    max_in_degree: Opt::Full(1),
                    max_out_degree: Opt::Full(1),
                    avg_out_degree: Opt::Full(0.98),
                    incoming_vert_count: Opt::Full(233),
                    outgoing_vert_count: Opt::Full(233)
                }];
        }

        describe! PtGraph {
            before_each {
                pub use super::super::*;
            }

            it "builds data1" {
                let (graph, number_of_read_bytes) = PtGraph::create("./tests/test_files/data1.txt".to_string());
                assert_eq!(number_of_read_bytes, _correct_read_bytes[0]);
                assert_eq!(_correct_stats[0], graph.stats());
            }

            it "builds data2" {
                let (graph, number_of_read_bytes) = PtGraph::create("./tests/test_files/data2.txt".to_string());
                assert_eq!(number_of_read_bytes, _correct_read_bytes[1]);
                assert_eq!(_correct_stats[1], graph.stats());
            }

            it "builds data3" {
                let (graph, number_of_read_bytes) = PtGraph::create("./tests/test_files/data3.txt".to_string());
                assert_eq!(number_of_read_bytes, _correct_read_bytes[2]);
                assert_eq!(_correct_stats[2], graph.stats());
            }

            // use catch_unwind as to not to poison global SEQUENCE mutex
            it "fails for input with too short read" {
                let result = panic::catch_unwind(|| {
                    HsGIR::create("./tests/test_files/data_too_short_read.txt".to_string());
                });
                assert!(result.is_err());
            }
        }
    }
}
