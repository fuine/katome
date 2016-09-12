//! Collection builder.
use data::primitives::Idx;
use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

extern crate bio;
use self::bio::io::fastq;

/// Description of how collection should be built.
pub trait Build : Default {
    /// Adds a single read to the collection.
    fn add_read(&mut self, read: &[u8]);

    /// Creates `GIR`/`Graph` from the supplied file,
    /// return with information about total number of read bytes.
    ///
    /// Currently supports fastaq format.
    fn create<P: AsRef<Path>>(path: P) -> (Self, usize) where Self: Sized {
        let mut total = 0usize;
        let mut collection = Self::default();
        let reader = match fastq::Reader::from_file(&path) {
            Err(why) => panic!("Couldn't open {}: {}", path.as_ref().display(), Error::description(&why)),
            Ok(lines) => lines,
        };
        info!("Starting to build collection");
        for sequence in reader.records() {
            let res = sequence.unwrap();
            let seq = res.seq();
            total += seq.len() as Idx;
            collection.add_read(seq);
        }
        info!("Collection built");
        (collection, total)
    }
}

// Count lines in the supplied file.
#[allow(dead_code)]
fn count_lines(filename: &str) -> usize {
    let file = File::open(filename).expect("I couldn't open that file, sorry :(");
    let reader = BufReader::new(file);
    reader.split(b'\n').count()
}
