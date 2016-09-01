use ::pbr::ProgressBar;
use data::primitives::Idx;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::io;

pub trait Build : Default {
    fn add_read(&mut self, read: &[u8]);

    /// Create `GIR`/`Graph` from the supplied fastaq file,
    /// return with information about total number of read bytes
    fn create(path: String, progress: bool) -> (Self, usize) where Self: Sized {
        let mut cnt = 0;
        let mut pb = ProgressBar::new(100 as u64);
        let chunk = if progress {
           let line_count = Self::count_lines(&path) / 4;
           pb.format("╢▌▌░╟");
           line_count / 100
        }
        else {
            0
        };
        let mut total = 0usize;
        let mut collection = Self::default();
        let mut lines = match Self::lines_from_file(&path) {
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
            if progress {
               cnt += 1;
               if cnt >= chunk {
                   cnt = 0;
                   pb.inc();
               }
            }
        }
        info!("Collection built");
        (collection, total)
    }

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
