
// open.rs
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::slice;
use std::boxed;
use data::types::{Sequence, Sequences, Graph, Edges,
                  VertexId, ReadSlice, K_SIZE};

// creates graph
pub fn read_sequences(path: String) -> Sequences{
    let mut sequences: Sequences = Vec::new();
    let mut lines = match lines_from_file(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path,
                                                   Error::description(&why)),
        Ok(lines) => lines,
    };
    loop {
        match lines.next() { // read line -- id
            None => { break },
            _ => {}
        }
        // TODO exit gracefully if format is wrong
        let sequence = lines.next().unwrap().unwrap();
        sequences.push(sequence.into_bytes());
        lines.next(); // read +
        lines.next(); // read quality
    }
    sequences
}

pub fn add_sequence_to_graph<'a>(vec: &'a Sequence, graph: &'a mut Graph, window_size: usize, counter: &mut u64) {
    let last_item = vec.windows(window_size).last().unwrap();
    // XXX iterates 2 times through the sequence
    for window in vec.windows(window_size){
        *counter += 1;
        if (window as *const _) != (last_item as *const _) {
            let from = ReadSlice::new(&window[0] as VertexId);
            let to = ReadSlice::new(&window[1] as VertexId);
            let mut found = true;
            match graph.get_mut(&from) {
                Some(edges) => modify_edge(edges, to),
                None => found = false,
            }
            if !found {
                graph.insert(from, Edges::new(to));
            }
        }
        else{
            let from = ReadSlice::new(&window[0] as VertexId);
            let mut found = true;
            match graph.get_mut(&from) {
                Some(edges) => {},
                None => found = false,
            }
            if !found {
                graph.insert(from, Edges::empty());
            }
        }
    }
}

fn modify_edge<'a>(edges: &mut Edges, to: ReadSlice){
    for i in edges.outgoing.iter_mut(){
        if i.0 == to{
            i.1 += 1;
            return
        }
    }
    edges.outgoing.push((to, 1));
}

fn lines_from_file<P>(filename: P) -> Result<io::Lines<io::BufReader<File>>, io::Error>
where P: AsRef<Path> {
    let file = try!(File::open(filename));
    Ok(io::BufReader::new(file).lines())
}

pub fn as_u8_slice(v: &u8, size: usize) -> &[u8] {
    unsafe{
        slice::from_raw_parts(v, size)
    }
}
