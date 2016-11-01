#![feature(plugin)]
#![plugin(stainless)]
#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
extern crate katome;
pub use katome::config::InputFileType;
pub use katome::algorithms::builder::Build;
pub use katome::asm::SEQUENCES;
pub use katome::asm::lock::LOCK;
pub use katome::collections::{HmGIR, HsGIR, PtGraph};
pub use katome::data::primitives::K_SIZE;
pub use katome::stats::{Counts, CollectionStats, Stats, Opt};
pub use std::sync::Mutex;
pub use std::panic;

/* macro_rules! t_as_expr {
    ($t:expr) => ($t);
}

macro_rules! test_build {
    ($($t:tt, $e:ty),*) => ($(
            describe! $e {
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
                    let _correct_read_bytes = vec![200, 9200, 23300];
                    let _correct_counts = vec![(62, 61), (5704, 5612), (14446, 14213)];
                    let filenames = vec![
                        "./tests/test_files/data1.txt".to_string(),
                        "./tests/test_files/data2.txt".to_string(),
                        "./tests/test_files/data3.txt".to_string(),
                        "./tests/test_files/data_too_short_reads".to_string(),
                    ];
                    let _correct_stats: Vec<CollectionStats> = _correct_counts.iter().map(|&(x, y)| CollectionStats::with_counts(x, y)).collect();
                }
                it "builds data1" {
                    let (col, number_of_read_bytes) = $t::create(filenames[0].clone(), InputFileType::Fastq);
                    assert_eq!(number_of_read_bytes, _correct_read_bytes[0]);
                    assert_eq!(_correct_stats[0], col.stats());
                }

                it "builds data2" {
                    let (col, number_of_read_bytes) = $t::create(filenames[1].clone(), InputFileType::Fastq);
                    assert_eq!(number_of_read_bytes, _correct_read_bytes[1]);
                    assert_eq!(_correct_stats[1], col.stats());
                }

                it "builds data3" {
                    let (col, number_of_read_bytes) = $t::create(filenames[2].clone(), InputFileType::Fastq);
                    assert_eq!(number_of_read_bytes, _correct_read_bytes[2]);
                    assert_eq!(_correct_stats[2], col.stats());
                }

                // use catch_unwind as to not to poison global SEQUENCE mutex
                it "fails for input with too short read" {
                    let result = panic::catch_unwind(|| {
                        $t::create(filenames[3].clone(), InputFileType::Fastq);
                    });
                    assert!(result.is_err());
                }
            }
    )*)
} */

describe! build {
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
        let _correct_read_bytes = vec![200, 9200, 23300];
        let _correct_counts = vec![(62, 61), (5704, 5612), (14446, 14213)];
        let filenames = vec![
            "./tests/test_files/data1.txt".to_string(),
            "./tests/test_files/data2.txt".to_string(),
            "./tests/test_files/data3.txt".to_string(),
            "./tests/test_files/data_too_short_reads".to_string(),
        ];
    }
    describe! girs {
        before_each {
            let _correct_stats: Vec<CollectionStats> = _correct_counts.iter().map(|&(x, y)| CollectionStats::with_counts(x, y)).collect();
        }

/* #[cfg(test)]
mod tests {
    pub use super::*;
    test_build!(HsGIR, HsGIR);
} */
        describe! HsGIR {
            before_each {
                pub use super::super::*;
            }

            it "builds data1" {
                let (gir, number_of_read_bytes) = HsGIR::create(filenames[0].clone(), InputFileType::Fastq);
                assert_eq!(number_of_read_bytes, _correct_read_bytes[0]);
                assert_eq!(_correct_stats[0], gir.stats());
            }

            it "builds data2" {
                let (gir, number_of_read_bytes) = HsGIR::create(filenames[1].clone(), InputFileType::Fastq);
                assert_eq!(number_of_read_bytes, _correct_read_bytes[1]);
                assert_eq!(_correct_stats[1], gir.stats());
            }

            it "builds data3" {
                let (gir, number_of_read_bytes) = HsGIR::create(filenames[2].clone(), InputFileType::Fastq);
                assert_eq!(number_of_read_bytes, _correct_read_bytes[2]);
                assert_eq!(_correct_stats[2], gir.stats());
            }

            // use catch_unwind as to not to poison global SEQUENCE mutex
            it "fails for input with too short read" {
                let result = panic::catch_unwind(|| {
                    HsGIR::create(filenames[3].clone(), InputFileType::Fastq);
                });
                assert!(result.is_err());
            }

        }
        describe! HmGIR {
            before_each {
                pub use super::super::*;
            }

            it "builds data1" {
                let (gir, number_of_read_bytes) = HmGIR::create(filenames[0].clone(), InputFileType::Fastq);
                assert_eq!(number_of_read_bytes, _correct_read_bytes[0]);
                assert_eq!(_correct_stats[0], gir.stats());
            }

            it "builds data2" {
                let (gir, number_of_read_bytes) = HmGIR::create(filenames[1].clone(), InputFileType::Fastq);
                assert_eq!(number_of_read_bytes, _correct_read_bytes[1]);
                assert_eq!(_correct_stats[1], gir.stats());
            }

            it "builds data3" {
                let (gir, number_of_read_bytes) = HmGIR::create(filenames[2].clone(), InputFileType::Fastq);
                assert_eq!(number_of_read_bytes, _correct_read_bytes[2]);
                assert_eq!(_correct_stats[2], gir.stats());
            }

            // use catch_unwind as to not to poison global SEQUENCE mutex
            it "fails for input with too short read" {
                let result = panic::catch_unwind(|| {
                    HsGIR::create(filenames[3].clone(), InputFileType::Fastq);
                });
                assert!(result.is_err());
            }
        }
    }
    describe! graphs {
        before_each {
            let _correct_stats = vec![
                CollectionStats {
                    capacity: (64, Opt::Full(64)),
                    counts: Counts {
                        node_count: _correct_counts[0].0,
                        edge_count: _correct_counts[0].1,
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
                        node_count: _correct_counts[1].0,
                        edge_count: _correct_counts[1].1,
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
                let (graph, number_of_read_bytes) = PtGraph::create(filenames[0].clone(), InputFileType::Fastq);
                assert_eq!(number_of_read_bytes, _correct_read_bytes[0]);
                assert_eq!(_correct_stats[0], graph.stats());
            }

            it "builds data2" {
                let (graph, number_of_read_bytes) = PtGraph::create(filenames[1].clone(), InputFileType::Fastq);
                assert_eq!(number_of_read_bytes, _correct_read_bytes[1]);
                assert_eq!(_correct_stats[1], graph.stats());
            }

            it "builds data3" {
                let (graph, number_of_read_bytes) = PtGraph::create(filenames[2].clone(), InputFileType::Fastq);
                assert_eq!(number_of_read_bytes, _correct_read_bytes[2]);
                assert_eq!(_correct_stats[2], graph.stats());
            }

            // use catch_unwind as to not to poison global SEQUENCE mutex
            it "fails for input with too short read" {
                let result = panic::catch_unwind(|| {
                    HsGIR::create(filenames[3].clone(), InputFileType::Fastq);
                });
                assert!(result.is_err());
            }
        }
    }
}
