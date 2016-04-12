extern crate pbr;
// open.rs
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::slice;
use std::cmp::{max};
use std::str;
use data::types::{Sequence, Sequences, Graph, Edges,
                  VertexId, ReadSlice, K_SIZE};
use self::pbr::{ProgressBar};
// creates graph
pub fn read_sequences(path: String, sequences: &mut Sequences, graph: &mut Graph) {
    // let mut sequences: Sequences = Vec::new();
    // let mut pb = ProgressBar::new(24294983 as u64);
    // let mut counter = 0;
    // pb.format("╢▌▌░╟");
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

        add_sequence_to_graph(&read, graph, sequences);
        lines.next(); // read +
        lines.next(); // read quality
        // pb.inc();
    }
}

pub fn add_sequence_to_graph<'a, 'b>(
        vec: &'a Sequence, graph: &'b mut Graph, reads: &'b mut Sequences){
    // XXX iterates 2 times through the read
    let last_item = vec.windows(K_SIZE).last().unwrap();
    // let mut inserted: Option<Box<Vec<u8>>> = None;
    let mut distance_counter = 0; // how many characters from the last inserted sequence in this read are we
    let mut cnt = K_SIZE;
    let total_size = vec.len();
    let mut window = vec.windows(K_SIZE);
    let mut is_first = true;
    let mut is_last = false;
    let mut from: Result<VertexId, (usize, usize)> = Err((0,0));
    let mut to: Option<Result<VertexId, (usize, usize)>> = Some(Err((0,0)));
    let mut window_value = window.next().unwrap();
    let mut read: Vec<Vec<u8>> = Vec::with_capacity(50);
    let mut vertices: Vec<(Result<VertexId, (usize, usize)>, Option<Result<VertexId, (usize, usize)>>)> = Vec::new();
    loop{
        if cnt == total_size -1 {
            is_last = true;
        }
        // Allocate proper memory
        if is_first {
            let from_tmp = ReadSlice::new(&window_value[0]);
            is_first = false;
            from = {
                match graph.get(&from_tmp) {
                    Some(edges) => {distance_counter +=1; Ok(edges.key)}, // guaranteed proper pointer
                    None => { // this sequence has not yet been inserted to the map -- we need to allocate space for it
                        if distance_counter == 0 || distance_counter > max(32, K_SIZE) { // no box for this read or number of bytes inserted is greater than size_of<Box<Vec<u8>>> + size_of<Vec<u8>>
                            distance_counter = 1;
                            new_box(window_value, &mut read);
                            Err((read.len() - 1, 0))
                        }
                        else {
                            //append distance_counter bytes to the last box in vector
                            // new_box(window.next().unwrap(), reads)
                            let id = read.len();
                            let mut last: &mut Vec<u8> = read.last_mut().unwrap();
                            let new_bytes: &[u8] = &window_value[K_SIZE - distance_counter..];
                            last.extend_from_slice(&new_bytes);
                            Err((id - 1, last.len()-K_SIZE))
                        }
                    }
                }
            }
        }
        else{
            from = to.unwrap();
            match from {
                _ => {},
            }
        }
        if !is_last{
            let to_tmp = ReadSlice::new(&window_value[1]);
            window_value = window.next().unwrap();
            to = {
                match graph.get(&to_tmp) {
                    Some(edges) => {distance_counter += 1; Some(Ok(edges.key))}, // guaranteed proper pointer
                    None => { // this sequence has not yet been inserted to the map -- we need to allocate space for it
                        if read.len() == 0 || distance_counter > max(32, K_SIZE) { // no box for this read or number of bytes inserted is greater than size_of<Box<Vec<u8>>> + size_of<Vec<u8>>
                            distance_counter = 1;
                            new_box(window_value, &mut read); // iterate here
                            Some(Err((read.len() - 1, 0)))
                        }
                        else {
                            //append distance_counter bytes to the last box in vector
                            let id = read.len() - 1;
                            let mut last: &mut Vec<u8> = read.last_mut().unwrap();
                            let new_bytes: &[u8] = &window_value[K_SIZE - distance_counter..];
                            last.extend_from_slice(&new_bytes);
                            Some(Err((id, last.len()-K_SIZE)))
                            // let len = last.len();
                            // &last[len-K_SIZE]
                        }
                    }
                }
            };
            vertices.push((from, to));
        }
        else{
            vertices.push((from, None));
        }
        // let ent = graph.entry(from).or_insert(Edges::new(from.ptr, to));
        // modify_edge(ent, to);
        cnt += 1;
        if is_last {
            break;
        }
    }
    let len = reads.len();
    for mut v in read {
        v.shrink_to_fit();
        reads.push(v);
    }

    for v in vertices {
        let from: ReadSlice = ReadSlice::new(match v.0 {
            Ok(x) => x,
            Err((v_, o)) => &(reads[len + v_])[o],
        });
        let to: Option<ReadSlice> =  match v.1 {
            Some(r) => {
                match r {
                    Ok(x) => Some(ReadSlice::new(x)),
                    Err((v_, o)) => Some(ReadSlice::new(&(reads[len + v_])[o])),
                }
            }
            None => None,
        };
        if let Some(r) = to {
        }
        let ent = graph.entry(from).or_insert(Edges::new(from.ptr, to));
        modify_edge(ent, to);
    }
}

fn new_box(window: &[u8], read: &mut Sequences) {
    let mut v_ = Vec::new();
    v_.extend_from_slice(window);
    read.push(v_);
}

fn modify_edge<'a>(edges: &mut Edges, to: Option<ReadSlice>){
    if let Some(to_) = to {
        for i in edges.outgoing.iter_mut(){
            if i.0 == to_{
                i.1 += 1;
                return
            }
        }
    }
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
