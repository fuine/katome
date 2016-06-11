// open.rs
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::slice;
use std::sync::Arc;
use std::cell::RefCell;
use data::edges::{Edges};
use std::collections::hash_map::Entry::*;
use data::read_slice::{ReadSlice};
use data::types::{Graph, VecArc,
                  VertexId, K_SIZE, Nodes};
// use asm::assembler::{VECTOR_RC};
use ::pbr::{ProgressBar};
// creates graph
pub fn read_sequences(path: String, sequences: VecArc, graph: &mut Graph,
                      in_nodes: &mut Nodes, out_nodes: &mut Nodes,
                      saved: &mut VertexId, total: &mut VertexId) {
    // let mut pb = ProgressBar::new(24294983 as u64);
    // let mut pb = ProgressBar::new(14214324 as u64);
    // pb.format("╢▌▌░╟");
    let mut lines = match lines_from_file(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path,
                                                   Error::description(&why)),
        Ok(lines) => lines,
    };
    let register: VecArc = Arc::new(RefCell::new(Vec::with_capacity(100))); // registry for a single line
    // TODO create new thread for in_ and out_ nodes
    // let mut cnt: u64 = 0;
    loop {
        if let None = lines.next() { break }  // read line -- id
        // TODO exit gracefully if format is wrong
        register.borrow_mut().clear(); // remove last line
        // XXX consider using append
        register.borrow_mut().extend_from_slice(lines.next().unwrap().unwrap().into_bytes().as_slice());
        *total += register.borrow().len() as VertexId;
        add_sequence_to_graph(register.clone(), graph, sequences.clone(), in_nodes, out_nodes, saved);
        lines.next(); // read +
        lines.next(); // read quality
        // pb.inc();
        // cnt += 1;
        // if cnt % 1000000 == 0 {
            // println!("{}: Graph has {} sequences and {} capacity", cnt, graph.len(), graph.capacity());
        // }
    }
    // TODO join on the in out thread
}

pub fn add_sequence_to_graph(
        vec: VecArc, graph: &mut Graph, reads: VecArc, in_nodes: &mut Nodes,
        out_nodes: &mut Nodes, saved: &mut VertexId) {
    assert!(vec.borrow().len() as VertexId >= K_SIZE + 1, "Read is too short!");
    let iterations = vec.borrow().len() - K_SIZE as usize;
    // let refcount: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(vec.clone()));
    let mut ins_counter: VertexId = 0;
    // let mut cnt: VertexId = 0;
    // let mut inserted = false;
    let mut index_counter = reads.borrow().len() as VertexId;
    let mut current: ReadSlice;
    // let mut previous: ReadSlice = ReadSlice::new(reads.clone(), index_counter);
    let mut insert = false;
    let mut prev_val_old: *mut Edges = 0 as *mut Edges;
    let mut prev_val_new: *mut Edges = 0 as *mut Edges;
    let mut is_in_node = false;
    let mut is_out_node = false;
    // let mut new_edge = false;
    for (cnt, window) in vec.borrow().windows(K_SIZE as usize).enumerate(){
        let from_tmp = ReadSlice::new(vec.clone(), cnt as VertexId);
        current = { // get a proper key to the hashmap
            match graph.entry(from_tmp) {
                Occupied(mut oe) => {
                    if ins_counter > 0 {
                        ins_counter += 1;
                    }
                    if cnt == 0 {
                        if oe.get().in_num == 0 {
                            is_in_node = true;
                        }
                    }
                    else if cnt == iterations && oe.get().outgoing.len() == 0 {
                        is_out_node = true;
                    }
                    prev_val_new = oe.get_mut() as *mut Edges;
                    oe.key().clone()
                }
                Vacant(_) => { // we cant use that VE because it is keyed with a temporary value
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
                    if cnt == 0 {
                        is_in_node = true;
                    }
                    if cnt == iterations {
                        is_out_node = true;
                    }
                    ins_counter = 1;
                    insert = true;
                    ReadSlice::new(reads.clone(), index_counter)
                }
            }
        };
        if cnt > 0 { // insert current sequence as a member of the previous
            let e: &mut Edges = unsafe {
                &mut *prev_val_old as &mut Edges
            };
            if modify_edge(e, current.offset) && !insert { // modify previous edge
                // new edge
                let cur: &mut Edges = unsafe {
                    &mut *prev_val_new as &mut Edges
                };
                cur.in_num += 1;
            }
        }
        if insert {
            let val_new = graph.entry(current.clone()).or_insert_with(Edges::empty);
            if cnt > 0 {
                val_new.in_num += 1;
            }
            prev_val_new = val_new as *mut Edges;
            insert = false;
        }
        if is_in_node { // add as input node
            in_nodes.insert(current.offset);
            is_in_node = false;
        }
        else if is_out_node { // add as output node
            out_nodes.insert(current.offset);
            is_out_node = false;
        }
        else { // remove from both input and output
            in_nodes.remove(&current.offset);
            out_nodes.remove(&current.offset);
        }
        prev_val_old = prev_val_new;
    }
}


fn modify_edge(edges: &mut Edges, to: VertexId) -> bool{
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
