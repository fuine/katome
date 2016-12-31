//! Configuration for the assembler.

use std::path::Path;

config_option_enum! {
    /// Format of the input file.
    InputFileType:
        /// Fasta format
        Fasta,
        /// Fastq format
        Fastq,
        /// BFCounter format
        BFCounter,
}

/// Config for assembler.
#[derive(Debug, RustcDecodable)]
pub struct Config<P: AsRef<Path>> {
    /// Paths of input files.
    pub input_files: Vec<P>,
    /// Type of the input file.
    pub input_file_type: InputFileType,
    /// Path to the output file.
    pub output_file: P,
    /// Length of the original (reference) genome.
    pub original_genome_length: usize,
    /// Minimal weight of the edge in de Bruijn graph.
    pub minimal_weight_threshold: usize,
    /// Size of the k-mer.
    pub k_mer_size: usize,
    /// Create reverse complements of the read sequences.
    ///
    ///  While this option noticeably slows down the process of assembly it
    ///  usually will create higher quality output. It is highly
    ///  advisable to use that option when using BFCounter file input.
    pub reverse_complement: bool,
}
