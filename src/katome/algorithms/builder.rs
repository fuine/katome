//! Collection builder.
use data::primitives::Idx;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::io;

/// Description of how collection should be built.
pub trait Build : Default {
    /// Adds a single read to the collection.
    fn add_read(&mut self, read: &[u8]);

    /// Creates `GIR`/`Graph` from the supplied file,
    /// return with information about total number of read bytes.
    ///
    /// Currently supports fastaq format.
    fn create(path: String) -> (Self, usize) where Self: Sized {
        let mut total = 0usize;
        let mut collection = Self::default();
        let mut lines = match lines_from_file(&path) {
            Err(why) => panic!("Couldn't open {}: {}", path, Error::description(&why)),
            Ok(lines) => lines,
        };
        let mut register = vec![];
        info!("Starting to build collection");
        loop {
            if let None = lines.next() { break; }  // read line -- id
            // TODO exit gracefully if format is wrong
            register.clear(); // remove last line
            // XXX consider using append
            register = lines.next().unwrap().unwrap().into_bytes();
            total += register.len() as Idx;
            collection.add_read(&register);
            lines.next(); // read +
            lines.next(); // read quality
        }
        info!("Collection built");
        (collection, total)
    }
}

fn lines_from_file(filename: &str) -> Result<io::Lines<io::BufReader<File>>, io::Error> {
    let file = try!(File::open(filename));
    Ok(io::BufReader::new(file).lines())
}

// Count lines in the supplied file.
#[allow(dead_code)]
fn count_lines(filename: &str) -> usize {
    let file = File::open(filename).expect("I couldn't open that file, sorry :(");
    let reader = BufReader::new(file);
    reader.split(b'\n').count()
}
