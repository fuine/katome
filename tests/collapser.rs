#[macro_use]
extern crate lazy_static;
extern crate katome;

pub use katome::config::InputFileType;
pub use katome::algorithms::builder::Build;
pub use katome::asm::SEQUENCES;
pub use katome::asm::lock::LOCK;
pub use katome::collections::{Convert, HmGIR, HsGIR, PtGraph};
pub use katome::prelude::{set_global_k_sizes};
pub use katome::algorithms::collapser::Collapsable;
pub use katome::stats::{Counts, CollectionStats, Stats, Opt};
pub use std::sync::Mutex;
pub use std::panic::catch_unwind;

macro_rules! before_each {
    ($l:ident, $f:ident, $c:ident) => {
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
        let $c = vec![2, 92, 233];
    }
}

macro_rules! collapses_data_graph {
    ($t: tt, $i: expr, $n: ident) => {
        #[test]
        fn $n() {
            let result = {
                before_each!(_l, filenames, lengths);
                catch_unwind(|| {
                    let (graph, _) = $t::create(&filenames[$i..$i+1], InputFileType::Fastq, false, 0);
                    let contigs = graph.collapse();
                    assert_eq!(contigs.len(), lengths[$i]);
                })
            };
            assert!(result.is_ok());
        }
    }
}

macro_rules! collapses_data_gir {
    ($t: tt, $g: tt, $i: expr, $n: ident) => {
        #[test]
        fn $n() {
            let result = {
                before_each!(_l, filenames, lengths);
                catch_unwind(|| {
                    let (gir, _) = $t::create(&filenames[$i..$i+1], InputFileType::Fastq, false, 0);
                    let graph = $g::create_from(gir);
                    let contigs = graph.collapse();
                    assert_eq!(contigs.len(), lengths[$i]);
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
            collapses_data_graph!($t, 0, collapses_data1);
            collapses_data_graph!($t, 1, collapses_data2);
            collapses_data_graph!($t, 2, collapses_data3);
        }
    }
}

macro_rules! test_gir {
    ($t:tt, $g:tt, $i:ident) => {
        mod $i {
            use super::*;
            collapses_data_gir!($t, $g, 0, collapses_data1);
            collapses_data_gir!($t, $g, 1, collapses_data2);
            collapses_data_gir!($t, $g, 2, collapses_data3);
        }
    }
}


#[cfg(test)]
mod collapse {
    pub use super::*;

    test_graph!(PtGraph, pt_graph);
    test_gir!(HsGIR, PtGraph, hs_gir);
    test_gir!(HmGIR, PtGraph, hm_gir);
}
