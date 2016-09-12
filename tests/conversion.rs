#![feature(plugin)]
#![plugin(stainless)]
#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
extern crate katome;

pub use katome::algorithms::builder::Build;
pub use katome::asm::assembler::lock::LOCK;
pub use katome::asm::assembler::{SEQUENCES};
pub use katome::data::collections::girs::Convert;
pub use katome::data::collections::girs::hm_gir::HmGIR;
pub use katome::data::collections::girs::hs_gir::{HsGIR};
pub use katome::data::collections::graphs::pt_graph::PtGraph;
pub use katome::data::primitives::K_SIZE;
pub use katome::data::statistics::{Counts, HasStats, Stats, Opt};
pub use std::sync::Mutex;

describe! conversion {
    before_each {
        // get global lock over sequences for testing
        let _l = LOCK.lock().unwrap();
        // Clear up SEQUENCES
        SEQUENCES.write().unwrap().clear();
        // hardcoded K_SIZE value for now :/
        assert_eq!(K_SIZE, 40);
        let _correct_counts = vec![(26, 26), (650, 650), (14446, 14213)];
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

    describe! to_PTGraph {

        describe! from_HsGIR {
            it "converts data1" {
                let (gir, _) = HsGIR::create("./tests/test_files/data1.txt".to_string());
                let gir_counts = gir.stats().counts;
                let graph = PtGraph::create_from(gir);
                assert_eq!(gir_counts, graph.stats().counts);
                assert_eq!(_correct_stats[0], graph.stats());
            }

            it "converts data2" {
                let (gir, _) = HsGIR::create("./tests/test_files/data2.txt".to_string());
                let gir_counts = gir.stats().counts;
                let graph = PtGraph::create_from(gir);
                assert_eq!(gir_counts, graph.stats().counts);
                assert_eq!(_correct_stats[1], graph.stats());
            }

            it "converts data3" {
                let (gir, _) = HsGIR::create("./tests/test_files/data3.txt".to_string());
                let gir_counts = gir.stats().counts;
                let graph = PtGraph::create_from(gir);
                assert_eq!(gir_counts, graph.stats().counts);
                assert_eq!(_correct_stats[2], graph.stats());
            }
        }

        describe! from_HmGIR {
            it "converts data1" {
                let (gir, _) = HmGIR::create("./tests/test_files/data1.txt".to_string());
                let gir_counts = gir.stats().counts;
                let graph = PtGraph::create_from(gir);
                assert_eq!(gir_counts, graph.stats().counts);
                assert_eq!(_correct_stats[0], graph.stats());
            }

            it "converts data2" {
                let (gir, _) = HmGIR::create("./tests/test_files/data2.txt".to_string());
                let gir_counts = gir.stats().counts;
                let graph = PtGraph::create_from(gir);
                assert_eq!(gir_counts, graph.stats().counts);
                assert_eq!(_correct_stats[1], graph.stats());
            }

            it "converts data3" {
                let (gir, _) = HmGIR::create("./tests/test_files/data3.txt".to_string());
                let gir_counts = gir.stats().counts;
                let graph = PtGraph::create_from(gir);
                assert_eq!(gir_counts, graph.stats().counts);
                assert_eq!(_correct_stats[2], graph.stats());
            }
        }
    }
}
