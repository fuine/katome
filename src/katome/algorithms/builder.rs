use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::io;

pub trait Build {
    /// Create `GIR`/`Graph` from the supplied fastaq file,
    /// return with information about total number of read bytes
    fn create(String) -> (Self, usize) where Self: Sized;

    fn lines_from_file(filename: &str) -> Result<io::Lines<io::BufReader<File>>, io::Error> {
        let file = try!(File::open(filename));
        Ok(io::BufReader::new(file).lines())
    }

    /// Count lines in the supplied file.
    #[allow(dead_code)]
    fn count_lines(filename: &str) -> usize {
        let file = File::open(filename).expect("I couldn't open that file, sorry :(");
        let reader = BufReader::new(file);
        reader.split(b'\n').count()
    }
}
