#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
extern crate katome;
pub use katome::config::InputFileType;
pub use katome::algorithms::builder::Build;
pub use katome::asm::SEQUENCES;
pub use katome::asm::lock::LOCK;
pub use katome::collections::{HmGIR, HsGIR, PtGraph};
pub use katome::prelude::set_global_k_sizes;
pub use katome::stats::{Counts, CollectionStats, Stats, Opt};
pub use std::sync::Mutex;
pub use std::panic::catch_unwind;

macro_rules! before_each {
    ($l:ident, $r:ident, $c:ident, $f:ident) => {
        // get global lock over sequences for testing
        let $l = LOCK.lock().unwrap();
        // Clear up SEQUENCES
        {
            let mut s = SEQUENCES.write();
            s.clear();
            s.push(vec![].into_boxed_slice());
        }
        unsafe { set_global_k_sizes(40); }
        let $r = vec![200, 9200, 23300];
        let $c = vec![(62, 61), (5704, 5612), (14446, 14213)];
        let $f = vec![
            "./tests/test_files/data1.txt".to_string(),
            "./tests/test_files/data2.txt".to_string(),
            "./tests/test_files/data3.txt".to_string(),
            "./tests/test_files/data_too_short_reads".to_string(),
        ];
    }
}

macro_rules! setup_gir {
    ($c:ident, $s:ident) => {
        let $s: Vec<CollectionStats> = $c.iter().map(|&(x, y)| CollectionStats::with_counts(x, y)).collect();
    }
}

macro_rules! setup_graph {
    ($c:ident, $s:ident) => {
        let $s = vec![
            CollectionStats {
                capacity: (64, Opt::Full(64)),
                counts: Counts {
                    node_count: $c[0].0,
                    edge_count: $c[0].1,
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
                capacity: (8192, Opt::Full(8192)),
                counts: Counts {
                    node_count: $c[1].0,
                    edge_count: $c[1].1,
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
                capacity: (16384, Opt::Full(16384)),
                counts: Counts {
                    node_count: $c[2].0,
                    edge_count: $c[2].1,
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
}

macro_rules! build_data_gir {
    ($t:tt, $i:expr, $n:ident) => {
        #[test]
        fn $n() {
            let result = {
                before_each!(_l, read_bytes, counts, filenames);
                catch_unwind(|| {
                    setup_gir!(counts, stats);
                    let (gir, number_of_read_bytes) = $t::create(filenames[$i].clone(), InputFileType::Fastq, false);
                    assert_eq!(number_of_read_bytes, read_bytes[$i]);
                    assert_eq!(stats[$i], gir.stats());
                })
            };
            assert!(result.is_ok());
        }
    }
}

macro_rules! build_data_graph {
    ($t:tt, $i:expr, $n:ident) => {
        #[test]
        fn $n() {
            let result = {
                before_each!(_l, read_bytes, _counts, filenames);
                catch_unwind(|| {
                    setup_graph!(_counts, stats);
                    let (graph, number_of_read_bytes) = PtGraph::create(filenames[$i].clone(), InputFileType::Fastq, false);
                    assert_eq!(number_of_read_bytes, read_bytes[$i]);
                    assert_eq!(stats[$i], graph.stats());
                })
            };
            assert!(result.is_ok());
        }
    }
}

macro_rules! fail_build {
    ($t:tt, $i:expr, $n:ident) => {
        #[test]
        fn $n() {
            before_each!(_l, _read_bytes, _counts, filenames);
            let result = catch_unwind(|| {
                $t::create(filenames[$i].clone(), InputFileType::Fastq, false);
            });
            assert!(result.is_err());
        }
    }
}

macro_rules! test_gir {
    ($t:tt, $i:ident) => {
        mod $i {
            use super::*;
            build_data_gir!($t, 0, builds0);
            build_data_gir!($t, 1, builds1);
            build_data_gir!($t, 2, builds2);
            fail_build!($t, 3, fails3);
        }
    }
}

macro_rules! test_graph {
    ($t:tt, $i:ident) => {
        mod $i {
            use super::*;
            build_data_graph!($t, 0, builds0);
            build_data_graph!($t, 1, builds1);
            build_data_graph!($t, 2, builds2);
            fail_build!($t, 3, fails3);
        }
    }
}

#[cfg(test)]
mod build {
    pub use super::*;
    test_gir!(HmGIR, hm_gir);
    test_gir!(HsGIR, hs_gir);
    test_graph!(PtGraph, pt_graph);
}
