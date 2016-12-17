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
    /// Path to input file.
    pub input_path: P,
    /// Type of the input file.
    pub input_file_type: InputFileType,
    /// Path to output file.
    pub output_path: P,
    /// Length of the original (reference) genome.
    pub original_genome_length: usize,
    /// Minimal weight of the edge in de Bruijn graph.
    pub minimal_weight_threshold: usize,
    /// Size of the k-mer.
    pub k_mer_size: usize,
}
