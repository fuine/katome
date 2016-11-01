//! De novo genome assemblers.
pub mod basic_assembler;

use config::Config;
use collections::{GIR, Graph, Convert};
use prelude::LockedSequences;
use algorithms::collapser::SerializedContigs;

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

pub struct Contigs {
    pub original_genome_length: usize,
    pub serialized_contigs: SerializedContigs,
}

impl Contigs {
    pub fn new(length_: usize, serialized: SerializedContigs) -> Contigs {
        Contigs {
            original_genome_length: length_,
            serialized_contigs: serialized,
        }
    }
}

/// Public API for assemblers.
pub trait Assemble {
    /// Assembles given data and writes results into the output file.
    fn assemble<P: AsRef<Path>, G: Graph>(config: Config<P>);

    /// Assembles given data and writes results into the output file.
    fn assemble_with_gir<P: AsRef<Path>, G, T: GIR>(config: Config<P>) where G: Graph + Convert<T>;
}
