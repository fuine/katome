#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]

#[macro_use]
extern crate lazy_static;
extern crate katome;

#[cfg(test)]
mod tests {
    pub use katome::data::gir::create_gir;
    pub use katome::data::graph::K_SIZE;
    pub use katome::asm::assembler::{SEQUENCES};
    pub use katome::asm::assembler::lock::LOCK;
    pub use katome::data::statistics::{HasStats, Stats};
    pub use std::sync::{Mutex};

    describe! integration {
        before_each {
            // get global lock over sequences for testing
            let _l = LOCK.lock().unwrap();
            // Clear up SEQUENCES
            SEQUENCES.write().unwrap().clear();
            // hardcoded K_SIZE value for now :/
            assert_eq!(K_SIZE, 40);
        }

        it "builds simplest gir" {
            let (gir, number_of_read_bytes) = create_gir("./tests/test_files/simplest_gir_creation.txt".to_string());
            let stats = gir.stats();
            let correct_stats = Stats::with_counts(26, 26);
            assert_eq!(number_of_read_bytes, 200);
            assert_eq!(correct_stats, stats);
        }

        it "builds simple gir" {
            let (gir, number_of_read_bytes) = create_gir("./tests/test_files/simple_gir_creation.txt".to_string());
            let stats = gir.stats();
            let correct_stats = Stats::with_counts(650, 650);
            assert_eq!(number_of_read_bytes, 2500);
            assert_eq!(correct_stats, stats);
        }
    }
}
