//! Collection builder.

use config::InputFileType;
use prelude::{EdgeWeight, Idx};


use std::error::Error;
use std::fs::{File, metadata, canonicalize};
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Result as Res;
use std::path::{Path, PathBuf};

extern crate bio;
use self::bio::io::{fasta, fastq};

/// Custom init function for collections
pub trait Init: Default {
    /// Initialize collection. Arguments are estimated maximum counts of nodes and
    /// edges, as well as type of the input file.
    fn init(_edges_count: Option<usize>, _nodes_count: Option<usize>, _ft: InputFileType) -> Self {
        Self::default()
    }
}

/// Description of how collection should be built.
pub trait Build: Init {
    /// Adds a single FASTA/FASTAQ read to the collection.
    fn add_read_fastaq(&mut self, read: &[u8], reverse_complement: bool);
    /// Adds a single BFCounter read to the collection.
    fn add_read_bfc(&mut self, _read: &[u8], _weight: EdgeWeight, _reverse_complement: bool) {
        // GIRs don't need to implement that function, as they serve the purpose
        // of doing what BFCounter does
        unreachable!()
    }

    /// Creates `GIR`/`Graph` from the supplied file,
    /// return with information about total number of read bytes.
    ///
    /// Currently supports fastaq format.
    fn create<P: AsRef<Path>>(input_files: &[P], ft: InputFileType, reverse_complement: bool,
                              minimal_weight_threshold: EdgeWeight)
                              -> (Self, usize)
        where Self: Sized {
        let files = check_files(input_files);
        match ft {
            InputFileType::Fasta => create_fasta(&files, reverse_complement),
            InputFileType::Fastq => create_fastq(&files, reverse_complement),
            InputFileType::BFCounter => {
                create_bfc(&files, reverse_complement, minimal_weight_threshold)
            }
        }
    }
}

fn check_files<P: AsRef<Path>>(input_files: &[P]) -> Vec<PathBuf> {
    let mut output = vec![];
    for file in input_files {
        let file = match canonicalize(file) {
            Ok(f) => f,
            Err(f) => panic!("Coulndt resolve path: {}", f.description()),
        };
        match metadata(&file) {
            Ok(attr) => {
                if attr.is_dir() {
                    panic!("{} is a directory", file.display());
                }
            }
            Err(_) => {
                panic!("{} does not exist", file.display());
            }
        };
        output.push(file);
    }
    output
}

fn create_bfc<T: Sized + Init + Build>(input_files: &[PathBuf], reverse_complement: bool,
                                       minimal_weight_threshold: EdgeWeight)
                                       -> (T, usize) {
    let mut total = 0_usize;
    let mut edge_count = 0_usize;
    for file in input_files {
        edge_count += count_lines(file);
    }
    let mut collection = T::init(Some(edge_count), None, InputFileType::BFCounter);
    let readers: Vec<_> = match input_files.iter().map(lines_from_file).collect() {
        Ok(r) => r,
        Err(why) => panic!("Couldn't open all files: {}", Error::description(&why)),
    };
    info!("Starting to build collection");
    for reader in readers {
        for e in reader {
            let e_ = e.unwrap();
            let mut iter = e_.split('\t');
            let edge = iter.next().unwrap().bytes().collect::<Vec<u8>>();
            let weight = match iter.next().unwrap().parse::<EdgeWeight>() {
                Ok(w) => w,
                Err(e) => {
                    panic!("Parse int error (if the kind is overflow user should change type of \
                            EdgeWeight in prelude.rs): {}",
                           e.description())
                }
            };
            if weight < minimal_weight_threshold {
                continue;
            }
            total += edge.len() as Idx;
            collection.add_read_bfc(&edge, weight, reverse_complement);
        }
    }
    info!("Collection built");
    (collection, total)
}

// TODO: remove nasty code duplication
fn create_fasta<T: Sized + Init + Build>(input_files: &[PathBuf], reverse_complement: bool)
                                         -> (T, usize) {
    let mut total = 0_usize;
    let mut collection = T::default();
    let readers: Vec<_> = match input_files.iter().map(fasta::Reader::from_file).collect() {
        Ok(r) => r,
        Err(why) => panic!("Couldn't open all files: {}", Error::description(&why)),
    };
    info!("Starting to build collection");
    for reader in readers {
        for sequence in reader.records() {
            let res = sequence.unwrap();
            let seq = res.seq();
            if !seq.iter().all(|&x| "ACGT".bytes().any(|i| i == x)) {
                continue;
            }
            total += seq.len() as Idx;
            collection.add_read_fastaq(seq, reverse_complement);
        }
    }
    info!("Collection built");
    (collection, total)
}

fn create_fastq<T: Sized + Init + Build>(input_files: &[PathBuf], reverse_complement: bool)
                                         -> (T, usize) {
    let mut total = 0_usize;
    let mut collection = T::default();
    let readers: Vec<_> = match input_files.iter().map(fastq::Reader::from_file).collect() {
        Ok(r) => r,
        Err(why) => panic!("Couldn't open all files: {}", Error::description(&why)),
    };
    info!("Starting to build collection");
    for (reader, filename) in readers.into_iter().zip(input_files.iter()) {
        for sequence in reader.records() {
            let res = sequence.unwrap();
            let seq = res.seq();
            if !seq.iter().all(|&x| "ACGT".bytes().any(|i| i == x)) {
                continue;
            }
            total += seq.len() as Idx;
            collection.add_read_fastaq(seq, reverse_complement);
        }
        info!("Done with {}", filename.display());
    }
    info!("Collection built");
    (collection, total)
}

fn lines_from_file<P: AsRef<Path>>(filename: P) -> Res<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

// Count lines in the supplied file.
#[allow(dead_code)]
fn count_lines<P: AsRef<Path>>(filename: &P) -> usize {
    let file = File::open(filename).expect("I couldn't open that file, sorry :(");
    let reader = BufReader::new(file);
    reader.split(b'\n').count()
}
