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
pub use katome::prelude::set_global_k_sizes;
pub use katome::stats::{Counts, Opt, CollectionStats, Stats};
pub use std::sync::Mutex;
pub use std::f64;
pub use std::panic::catch_unwind;

macro_rules! before_each {
    ($l:ident, $s:ident, $f:ident) => {
        // get global lock over sequences for testing
        let $l = LOCK.lock().unwrap();
        // Clear up SEQUENCES
        {
            let mut s = SEQUENCES.write();
            s.clear();
            s.push(vec![].into_boxed_slice());
        }
        unsafe { set_global_k_sizes(40); }
        let $f = vec![
            "./tests/test_files/data1.txt".to_string(),
            "./tests/test_files/data2.txt".to_string(),
            "./tests/test_files/data3.txt".to_string(),
        ];
        let $s = vec![
            vec![
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
            ],
            vec![
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
                }
            ],
            vec![
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
            ],
        ];
    }
}

macro_rules! test_graph_on_data {
    ($t:tt, $n:ident, $i:expr, $w:expr) => {
        mod $n {
            use super::*;
            #[test]
            fn removes_single_vertices() {
                // TODO remove some edges to show that vertices will get removed?
                let result = {
                    before_each!(_l, stats, filenames);
                    catch_unwind(|| {
                        let (mut graph, _) = $t::create(filenames[$i].clone(), InputFileType::Fastq, false, 0);
                        graph.remove_single_vertices();
                        assert_eq!(stats[$i][0], graph.stats());
                    })
                };
                assert!(result.is_ok());
            }

            #[test]
            fn removes_weak_edges() {
                let result = {
                    before_each!(_l, stats, filenames);
                    catch_unwind(|| {
                        let (mut graph, _) = $t::create(filenames[$i].clone(), InputFileType::Fastq, false, 0);
                        graph.remove_weak_edges($w);
                        assert_eq!(stats[$i][1].counts, graph.stats().counts);
                    })
                };
                assert!(result.is_ok());
            }

            #[test]
            fn removes_dead_paths() {
                let result = {
                    before_each!(_l, stats, filenames);
                    catch_unwind(|| {
                        let (mut graph, _) = $t::create(filenames[$i].clone(), InputFileType::Fastq, false, 0);
                        graph.remove_dead_paths();
                        assert_eq!(stats[$i][2].counts, graph.stats().counts);
                    })
                };
                assert!(result.is_ok());
            }
        }
    }
}

macro_rules! test_gir_on_data {
    ($t:tt, $n:ident, $g:tt, $i:expr, $w:expr) => {
        mod $n {
            use super::*;
            #[test]
            fn removes_single_vertices() {
                // TODO remove some edges to show that vertices will get removed?
                let result = {
                    before_each!(_l, stats, filenames);
                    catch_unwind(|| {
                        let (mut gir, _) = $t::create(filenames[$i].clone(), InputFileType::Fastq, false, 0);
                        gir.remove_single_vertices();
                        assert_eq!(stats[$i][0].counts, gir.stats().counts);
                        let graph = $g::create_from(gir);
                        assert_eq!(stats[$i][0], graph.stats());
                    })
                };
                assert!(result.is_ok());
            }

            #[test]
            fn removes_weak_edges() {
                let result = {
                    before_each!(_l, stats, filenames);
                    catch_unwind(|| {
                        let (mut gir, _) = $t::create(filenames[$i].clone(), InputFileType::Fastq, false, 0);
                        gir.remove_weak_edges($w);
                        assert_eq!(stats[$i][1].counts, gir.stats().counts);
                    })
                };
                assert!(result.is_ok());
            }
        }
    }
}

macro_rules! test_gir {
    ($t:tt, $n:ident, $g:tt) => {
        mod $n {
            pub use super::*;
            test_gir_on_data!($t, data_1, $g, 0, 3);
            // test_gir_on_data!($t, data_2, $g, 1, 3);
            test_gir_on_data!($t, data_3, $g, 2, 1);
        }
    }
}

macro_rules! test_graph {
    ($t:tt, $n:ident) => {
        mod $n {
            pub use super::*;
            test_graph_on_data!($t, data_1, 0, 3);
            test_graph_on_data!($t, data_2, 1, 2);
            test_graph_on_data!($t, data_3, 2, 1);
        }
    }
}

#[cfg(test)]
mod prune {
    pub use super::*;
    test_gir!(HmGIR, hm_gir, PtGraph);
    test_graph!(PtGraph, pt_graph);
}
