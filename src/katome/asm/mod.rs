//! De novo genome assemblers.
pub mod basic_assembler;

use data::primitives::LockedSequences;
use std::path::Path;
lazy_static! {
    /// Global mutable vector of bytes. Contains unique reads slices (k-mers).
    ///
    /// `ReadSlice` uses offsets on this structure to efficiently store
    /// information about sequence. Global container allows to save 8 bytes in
    /// ReadSlice (it doesn't have to store `Arc` to the container).
    pub static ref SEQUENCES: LockedSequences = LockedSequences::default();
}

#[doc(hidden)]
pub mod lock {
    use std::sync::Mutex;
    // mutex over sequences specifically for tests
    lazy_static! {
        pub static ref LOCK: Mutex<()> = Mutex::new(());
    }
}

/// Public API for assemblers.
pub trait Assemble {
    /// Assembles given data and writes results into the output file.
    ///
    /// * `input` - Fastaq or fasta input file.
    /// * `output` - Path to the output file. Each line in the output file denotes single contig.
    /// * `original_genome_length` - length of the reference genome.
    /// * `minimal_weight_threshold` - threshold used by `Pruner`.
    fn assemble<P: AsRef<Path>>(input: P, _output: P, original_genome_length: usize,
                minimal_weight_threshold: usize);
}
