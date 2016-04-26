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
// use ::pbr::{ProgressBar};
// creates graph
pub fn read_sequences(path: String, sequences: VecArc, graph: &mut Graph,
                      in_nodes: &mut Nodes, out_nodes: &mut Nodes,
                      saved: &mut usize, total: &mut usize) {
    // let mut pb = ProgressBar::new(24294983 as u64);
    // let mut counter = 0;
    // pb.format("╢▌▌░╟");
    let mut lines = match lines_from_file(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path,
                                                   Error::description(&why)),
        Ok(lines) => lines,
    };
    let register: VecArc = Arc::new(RefCell::new(Vec::with_capacity(100))); // registry for a single line
    // TODO create new thread for in_ and out_ nodes
    loop {
        match lines.next() { // read line -- id
            None => { break },
            _ => {}
        }
        // TODO exit gracefully if format is wrong
        register.borrow_mut().clear(); // remove last line
        // XXX consider using append
        register.borrow_mut().extend_from_slice(lines.next().unwrap().unwrap().into_bytes().as_slice());
        *total += register.borrow().len();
        add_sequence_to_graph(register.clone(), graph, sequences.clone(), in_nodes, out_nodes, saved);
        lines.next(); // read +
        lines.next(); // read quality
        // pb.inc();
    }
    // TODO join on the in out thread
}

pub fn add_sequence_to_graph(
        vec: VecArc, graph: &mut Graph, reads: VecArc, in_nodes: &mut Nodes,
        out_nodes: &mut Nodes, saved: &mut usize) {
    assert!(vec.borrow().len() >= K_SIZE + 1, "Read is too short!");
    let iterations = vec.borrow().len() - K_SIZE;
    // let refcount: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(vec.clone()));
    let mut ins_counter = 0;
    let mut cnt = 0;
    // let mut inserted = false;
    let mut index_counter = reads.borrow().len();
    let mut current: ReadSlice = ReadSlice::new(reads.clone(), index_counter);
    // let mut previous: ReadSlice = ReadSlice::new(reads.clone(), index_counter);
    let mut insert = false;
    let mut prev_val_old: *mut Edges = 0 as *mut Edges;
    let mut prev_val_new: *mut Edges = 0 as *mut Edges;
    let mut is_in_node = false;
    let mut is_out_node = false;
    for window in vec.borrow().windows(K_SIZE){
        let from_tmp = ReadSlice::new(vec.clone(), cnt);
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
                    else if cnt == iterations {
                        if oe.get().out_num == 0 {
                            is_out_node = true;
                        }
                    }
                    if cnt > 0 {
                        oe.get_mut().in_num += 1;
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
                        reads.borrow_mut().extend_from_slice(&window[K_SIZE - ins_counter ..]);
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
            modify_edge(e, current.offset); // modify previous edge
        }
        if insert {
            let val_new = graph.entry(current.clone()).or_insert(Edges::empty());
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
        // previous = current;
        cnt += 1;
    }
}


fn modify_edge<'a>(edges: &mut Edges, to: VertexId){
    for i in edges.outgoing.iter_mut(){
        if i.0 == to {
            i.1 += 1;
            edges.out_num += 1;
            return
        }
    }
    edges.outgoing.push((to, 1));
    edges.out_num += 1;
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
