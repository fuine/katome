#[macro_use]
extern crate lazy_static;
extern crate katome;

pub use katome::config::InputFileType;
pub use katome::algorithms::builder::Build;
pub use katome::asm::SEQUENCES;
pub use katome::asm::lock::LOCK;
pub use katome::collections::{Convert, HmGIR, HsGIR, PtGraph};
pub use katome::collections::graphs::pt_graph::write_to_dot;
pub use katome::prelude::K_SIZE;
pub use katome::algorithms::shrinker::Shrinkable;
pub use katome::stats::{Counts, CollectionStats, Stats, Opt};
pub use std::sync::Mutex;
pub use std::panic::catch_unwind;

macro_rules! before_each {
    ($l:ident, $f:ident, $c:ident, $p:ident) => {
        // get global lock over sequences for testing
        let $l = LOCK.lock().unwrap();
        // Clear up SEQUENCES
        {
            let mut s = SEQUENCES.write();
            s.clear();
            s.push(vec![].into_boxed_slice());
        }
        // hardcoded K_SIZE value for now :/
        assert_eq!(K_SIZE, 40);
        let $f = vec![
            "./tests/test_files/data1.txt".to_string(),
            "./tests/test_files/data2.txt".to_string(),
            "./tests/test_files/data3.txt".to_string(),
        ];
        let pre = vec![(62, 61), (5704, 5612), (14446, 14213)];
        let post = vec![
            (2, 1), (184, 92), (466, 233),
        ];
        let $c = pre.iter().map(|&(x, y)| Counts{ node_count: x, edge_count: y }).collect::<Vec<_>>();
        let $p = post.iter().map(|&(x, y)| Counts{ node_count: x, edge_count: y }).collect::<Vec<_>>();
    }
}

macro_rules! shrinks_graph {
    ($t: tt, $i: expr, $n: ident) => {
        #[test]
        fn $n() {
            let result = {
                before_each!(_l, filenames, counts_pre, counts_post);
                catch_unwind(|| {
                    let (mut graph, _) = $t::create(filenames[$i].clone(), InputFileType::Fastq);
                    assert_eq!(graph.stats().counts, counts_pre[$i]);
                    graph.shrink();
                    assert_eq!(graph.stats().counts, counts_post[$i]);
                })
            };
            assert!(result.is_ok());
        }
    }
}

macro_rules! test_graph {
    ($t:tt, $i:ident) => {
        mod $i {
            use super::*;
            shrinks_graph!($t, 0, shrinks_data1);
            shrinks_graph!($t, 1, shrinks_data2);
            shrinks_graph!($t, 2, shrinks_data3);
        }
    }
}

#[cfg(test)]
mod shrink {
    pub use super::*;

    test_graph!(PtGraph, pt_graph);
}
