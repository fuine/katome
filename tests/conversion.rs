#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
extern crate katome;

pub use katome::config::InputFileType;
pub use katome::algorithms::builder::Build;
pub use katome::asm::SEQUENCES;
pub use katome::asm::lock::LOCK;
pub use katome::collections::{Convert, HmGIR, HsGIR, PtGraph};
pub use katome::prelude::set_global_k_sizes;
pub use katome::stats::{Counts, CollectionStats, Stats, Opt};
pub use std::sync::Mutex;
pub use std::panic::catch_unwind;


macro_rules! before_each {
    ($l:ident, $f:ident, $s:ident) => {
        // get global lock over sequences for testing
        let $l = LOCK.lock().unwrap();
        // Clear up SEQUENCES
        {
            let mut s = SEQUENCES.write();
            s.clear();
            s.push(vec![].into_boxed_slice());
        }
        unsafe { set_global_k_sizes(40); }
        let counts = vec![(62, 61), (5704, 5612), (14446, 14213)];
        let $f = vec![
            "./tests/test_files/data1.txt".to_string(),
            "./tests/test_files/data2.txt".to_string(),
            "./tests/test_files/data3.txt".to_string(),
        ];
        let $s = vec![
            CollectionStats {
                capacity: (64, Opt::Full(64)),
                counts: Counts {
                    node_count: counts[0].0,
                    edge_count: counts[0].1,
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
                    node_count: counts[1].0 ,
                    edge_count: counts[1].1
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
                    node_count: counts[2].0,
                    edge_count: counts[2].1,
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

macro_rules! converts_gir {
    ($t:tt, $g:tt, $i:expr, $n:ident) => {
        #[test]
        fn $n() {
            let result = {
                before_each!(_l, filenames, stats);
                catch_unwind(|| {
                    let (gir, _) = $t::create(&filenames[$i..$i+1], InputFileType::Fastq, false, 0);
                    let gir_counts = gir.stats().counts;
                    let graph = $g::create_from(gir);
                    assert_eq!(gir_counts, graph.stats().counts);
                    assert_eq!(stats[$i], graph.stats());
                })
            };
            assert!(result.is_ok());
        }
    }
}

macro_rules! test_gir {
    ($t:tt, $g:tt, $i:ident) => {
        mod $i {
            use super::*;
            converts_gir!($t, $g, 0, converts1);
            converts_gir!($t, $g, 1, converts2);
            converts_gir!($t, $g, 2, converts3);
        }
    }
}

#[cfg(test)]
mod conversion {
    pub use super::*;
    test_gir!(HsGIR, PtGraph, hs_gir);
    test_gir!(HmGIR, PtGraph, hm_gir);
}
