//! De novo genome assemblers.
pub mod basic_assembler;

use algorithms::builder::InputFileType;
use data::collections::girs::{GIR, Convert};
use data::collections::graphs::Graph;
use data::primitives::LockedSequences;
use std::path::Path;
lazy_static! {
    /// Global mutable vector of bytes. Contains unique reads slices (k-mers).
    ///
    /// `ReadSlice` uses offsets on this structure to efficiently store
    /// information about sequence. Global container allows to save 8 bytes in
    /// ReadSlice (it doesn't have to store `Arc` to the container).
    pub static ref SEQUENCES: LockedSequences = {
        let l = LockedSequences::default();
        l.write().push(vec![].into_boxed_slice());
        l
    };
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
    fn assemble<P: AsRef<Path>, G: Graph>(input: P, _output: P, original_genome_length: usize,
                                          minimal_weight_threshold: usize, ft: InputFileType);

    /// Assembles given data and writes results into the output file.
    ///
    /// * `input` - Fastaq or fasta input file.
    /// * `output` - Path to the output file. Each line in the output file denotes single contig.
    /// * `original_genome_length` - length of the reference genome.
    /// * `minimal_weight_threshold` - threshold used by `Pruner`.
    fn assemble_with_gir<P: AsRef<Path>, G, T: GIR>(input: P, _output: P,
                                                    original_genome_length: usize,
                                                    minimal_weight_threshold: usize,
                                                    ft: InputFileType)
        where G: Graph + Convert<T>;
}
