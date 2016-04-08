
// open.rs
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::slice;
use std::boxed;
use data::types::{Sequence, Sequences, Graph, Edges,
                  VertexId, ReadSlice, K_SIZE, ReadPtr};

// creates graph
pub fn read_sequences(path: String, sequences: &mut Sequences, graph: &mut Graph) {
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
        let read = lines.next().unwrap().unwrap().into_bytes();
        // let sq = sequence.into_bytes();

        if let Some(bx) = add_sequence_to_graph(&read, graph) {
            sequences.push(bx);
        }
        lines.next(); // read +
        lines.next(); // read quality
    }
}

pub fn add_sequence_to_graph<'a>(
        vec: &'a Sequence, graph: &'a mut Graph) -> Option<ReadPtr>{
    // XXX iterates 2 times through the read
    let last_item = vec.windows(K_SIZE).last().unwrap();
    let mut inserted: Option<Box<Vec<u8>>> = None;
    for window in vec.windows(K_SIZE){
        if (window as *const _) != (last_item as *const _) {
            let from = ReadSlice::new(&window[0] as VertexId);
            let to = ReadSlice::new(&window[1] as VertexId);
            let mut found = true;
            match graph.get_mut(&from) {
                Some(edges) => modify_edge(edges, to),
                None => found = false,
            }
            if !found { // we need to insert a new sequence and keep it's pointer valid
                let ptr: VertexId = match inserted {
                    Some(ref seq) => &(**seq)[0], //unwrap ref to box and then box itself
                    None      => {
                        let s: ReadPtr = Box::new(vec.clone());  // sequence is on the heap now
                        let b: VertexId = &(*s)[0];
                        inserted = Some(s);
                        b
                    }
                };
                // let ptr: *const u8 = &(*seq)[0];
                graph.insert(from, Edges::new(ReadSlice::new(ptr)));
            }
        }
    }
    inserted
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
