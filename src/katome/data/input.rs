// input.rs
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::slice;
use std::sync::Arc;
use std::cell::RefCell;
use data::edges::{Edges};
use std::collections::hash_map::{Entry};
use data::read_slice::ReadSlice;
use data::types::{Graph, VecArc,
                  VertexId, K_SIZE};
use std::io::BufReader;
use ::pbr::{ProgressBar};

// creates graph
pub fn read_sequences(path: String, sequences: VecArc, graph: &mut Graph,
                      saved: &mut VertexId, total: &mut usize, number_of_reads: &mut usize) {
    let line_count = count_lines(&path) / 4;
    let chunk = line_count / 100;
    let mut cnt = 0;
    // let mut pb = ProgressBar::new(24294983 as u64);
    let mut pb = ProgressBar::new(100 as u64);
    pb.format("╢▌▌░╟");
    let mut lines = match lines_from_file(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path,
                                                   Error::description(&why)),
        Ok(lines) => lines,
    };
    let register: VecArc = Arc::new(RefCell::new(Vec::with_capacity(100))); // register for a single line
    loop {
        if let None = lines.next() { break }  // read line -- id
        // TODO exit gracefully if format is wrong
        register.borrow_mut().clear(); // remove last line
        // XXX consider using append
        register.borrow_mut().extend_from_slice(lines.next().unwrap().unwrap().into_bytes().as_slice());
        *total += register.borrow().len() as VertexId;
        add_sequence_to_graph(register.clone(), graph, sequences.clone(), saved);
        lines.next(); // read +
        lines.next(); // read quality
        *number_of_reads += 1;
        cnt += 1;
        if cnt >= chunk {
            cnt = 0;
            pb.inc();
        }
    }
}

pub fn add_sequence_to_graph(
        vec: VecArc, graph: &mut Graph, reads: VecArc, saved: &mut VertexId) {
    assert!(vec.borrow().len() as VertexId >= K_SIZE + 1, "Read is too short!");
    let mut ins_counter: VertexId = 0;
    let mut index_counter = reads.borrow().len() as VertexId;
    let mut current: ReadSlice;
    let mut insert = false;
    let mut previous_node: ReadSlice = RS!(vec, 0);
    // let mut prev_val_old: *mut Edges = 0 as *mut Edges;
    let mut prev_val_new: *mut Edges = 0 as *mut Edges;
    for (cnt, window) in vec.borrow().windows(K_SIZE as usize).enumerate(){
        let from_tmp = RS!(vec, cnt as VertexId);
        current = { // get a proper key to the hashmap
            match graph.entry(from_tmp) {
                Entry::Occupied(mut oe) => {
                    if ins_counter > 0 {
                        ins_counter += 1;
                    }
                    prev_val_new = oe.get_mut() as *mut Edges;
                    oe.key().clone()
                }
                Entry::Vacant(_) => { // we cant use that VE because it is keyed with a temporary value
                    // push to vector
                    if ins_counter == 0 {
                        // append window to vector
                        reads.borrow_mut().extend_from_slice(window);
                        *saved += K_SIZE;
                    }
                    else if ins_counter > K_SIZE {
                        // append window to vector
                        reads.borrow_mut().extend_from_slice(window);
                        index_counter += K_SIZE;
                        *saved += K_SIZE;
                    }
                    else {
                        // append only ins_counter last bytes of window
                        reads.borrow_mut().extend_from_slice(&window[(K_SIZE - ins_counter ) as usize ..]);
                        index_counter += ins_counter;
                        *saved += ins_counter;
                    }
                    ins_counter = 1;
                    insert = true;
                    RS!(reads, index_counter)
                }
            }
        };
        if cnt > 0 { // insert current sequence as a member of the previous
            let modified: bool;
            {
                let e: &mut Edges = graph.get_mut(&previous_node).unwrap();
                modified = modify_edge(e, current.offset);
            }
            if modified && !insert { // modify previous edge
                // new edge
                let cur: &mut Edges = unsafe {
                    &mut *prev_val_new as &mut Edges
                };
                cur.in_num += 1;
            }
        }
        if insert {
            let val_new: &mut Edges = graph.entry(current.clone()).or_insert_with(Edges::empty);
            if cnt > 0 {
                val_new.in_num += 1;
            }
            insert = false;
        }
        previous_node = current;
    }
}

fn count_lines(filename: &str) -> usize {
    let file = File::open(filename).expect("I couldn't open that file, sorry :(");

    let reader = BufReader::new(file);

    reader.split(b'\n').count()
}

fn modify_edge(edges: &mut Edges, to: VertexId) -> bool {
    for i in edges.outgoing.iter_mut(){
        if i.0 == to {
            i.1 += 1;
            return false;
        }
    }
    let mut out_ = Vec::new();
    out_.extend_from_slice(&edges.outgoing);
    out_.push((to, 1));
    edges.outgoing = out_.into_boxed_slice();
    true
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
