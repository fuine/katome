//! Collection builder.

use config::InputFileType;
use prelude::{EdgeWeight, Idx};


use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Result as Res;
use std::path::Path;

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
    fn add_read_fastaq(&mut self, read: &[u8]);
    /// Adds a single BFCounter read to the collection.
    fn add_read_bfc(&mut self, _read: &[u8], _weight: EdgeWeight) {
        // GIRs don't need to implement that function, as they serve the purpose
        // of doing what BFCounter does
        unreachable!()
    }

    /// Creates `GIR`/`Graph` from the supplied file,
    /// return with information about total number of read bytes.
    ///
    /// Currently supports fastaq format.
    fn create<P: AsRef<Path>>(path: P, ft: InputFileType) -> (Self, usize) where Self: Sized {
        match ft {
            InputFileType::Fasta => create_fasta(path),
            InputFileType::Fastq => create_fastq(path),
            InputFileType::BFCounter => create_bfc(path),
        }
    }
}

fn create_bfc<P: AsRef<Path>, T: Sized + Init + Build>(path: P) -> (T, usize) {
    let mut total = 0usize;
    let edge_count = count_lines(&path);
    let mut collection = T::init(Some(edge_count), None, InputFileType::BFCounter);
    let reader = match lines_from_file(&path) {
        Err(why) => {
            panic!("Couldn't open {}: {}", path.as_ref().display(), Error::description(&why))
        }
        Ok(lines) => lines,
    };
    info!("Starting to build collection");
    for e in reader {
        let e_ = e.unwrap();
        let mut iter = e_.split('\t');
        let edge = iter.next().unwrap().bytes().collect::<Vec<u8>>();
        let weight = iter.next().unwrap().parse::<EdgeWeight>().unwrap();
        total += edge.len() as Idx;
        collection.add_read_bfc(&edge, weight);
    }
    info!("Collection built");
    (collection, total)
}

// TODO: remove nasty code duplication
fn create_fasta<P: AsRef<Path>, T: Sized + Init + Build>(path: P) -> (T, usize) {
    let mut total = 0usize;
    let mut collection = T::default();
    let reader = match fasta::Reader::from_file(&path) {
        Err(why) => {
            panic!("Couldn't open {}: {}", path.as_ref().display(), Error::description(&why))
        }
        Ok(lines) => lines,
    };
    info!("Starting to build collection");
    for sequence in reader.records() {
        let res = sequence.unwrap();
        let seq = res.seq();
        if !seq.iter().all(|&x| "ACGT".bytes().any(|i| i == x)) {
            continue;
        }
        total += seq.len() as Idx;
        collection.add_read_fastaq(seq);
    }
    info!("Collection built");
    (collection, total)
}

fn create_fastq<P: AsRef<Path>, T: Sized + Init + Build>(path: P) -> (T, usize) {
    let mut total = 0usize;
    let mut collection = T::default();
    let reader = match fastq::Reader::from_file(&path) {
        Err(why) => {
            panic!("Couldn't open {}: {}", path.as_ref().display(), Error::description(&why))
        }
        Ok(lines) => lines,
    };
    info!("Starting to build collection");
    for sequence in reader.records() {
        let res = sequence.unwrap();
        let seq = res.seq();
        if !seq.iter().all(|&x| "ACGT".bytes().any(|i| i == x)) {
            continue;
        }
        total += seq.len() as Idx;
        collection.add_read_fastaq(seq);
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
