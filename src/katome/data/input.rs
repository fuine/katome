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
use data::types::{Sequences, Graph, VecArc, // VecRcPtr,
                  VertexId, K_SIZE};
// use asm::assembler::{VECTOR_RC};
// use ::pbr::{ProgressBar};
// creates graph
pub fn read_sequences(path: String, sequences: VecArc, graph: &mut Graph) {
    // let mut pb = ProgressBar::new(24294983 as u64);
    // let mut counter = 0;
    // pb.format("╢▌▌░╟");
    let mut lines = match lines_from_file(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path,
                                                   Error::description(&why)),
        Ok(lines) => lines,
    };
    let register: VecArc = Arc::new(RefCell::new(Vec::with_capacity(100))); // registry for a single line
    loop {
        match lines.next() { // read line -- id
            None => { break },
            _ => {}
        }
        // TODO exit gracefully if format is wrong
        register.borrow_mut().clear(); // remove last line
        // XXX consider using append
        register.borrow_mut().extend_from_slice(lines.next().unwrap().unwrap().into_bytes().as_slice());
        add_sequence_to_graph(register.clone(), graph, sequences.clone());
        lines.next(); // read +
        lines.next(); // read quality
        // pb.inc();
    }
}

pub fn add_sequence_to_graph(
        vec: VecArc, graph: &mut Graph, reads: VecArc) {
    assert!(vec.borrow().len() >= K_SIZE + 1, "Read is too short!");
    // let iterations = vec.borrow().len() - K_SIZE;
    // let refcount: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(vec.clone()));
    let mut ins_counter = 0;
    let mut cnt = 0;
    // let mut inserted = false;
    let mut index_counter = reads.borrow().len();
    let mut current: ReadSlice = ReadSlice::new(reads.clone(), index_counter);
    let mut previous: ReadSlice = ReadSlice::new(reads.clone(), index_counter);
    let mut insert = false;
    for window in vec.borrow().windows(K_SIZE){
        let from_tmp = ReadSlice::new(vec.clone(), cnt);
        current = {
            match graph.entry(from_tmp) {
                Occupied(oe) => {
                    if ins_counter > 0 {
                        ins_counter += 1;
                    }
                    oe.key().clone()
                }
                Vacant(_) => { // we cant use that VE because it is keyed with a temporary value
                    // push to vector
                    if ins_counter == 0 {
                        // append window to vector
                        reads.borrow_mut().extend_from_slice(window);
                    }
                    else if ins_counter > K_SIZE {
                        // append window to vector
                        reads.borrow_mut().extend_from_slice(window);
                        index_counter += K_SIZE;
                    }
                    else {
                        // append only ins_counter last bytes of window
                        reads.borrow_mut().extend_from_slice(&window[K_SIZE - ins_counter ..]);
                        index_counter += ins_counter;
                    }
                    ins_counter = 1;
                    insert = true;
                    ReadSlice::new(reads.clone(), index_counter)
                }
            }
        };
        if insert {
            graph.insert(current.clone(), Edges::empty());
            insert = false;
        }
        if cnt > 0 { // insert current sequence as a member of the previous
            modify_edge(graph.get_mut(&previous).unwrap(), current.offset);
        }
        previous = current.clone();
        // XXX
        cnt += 1;
    }
}


fn modify_edge<'a>(edges: &mut Edges, to: VertexId){
    for i in edges.outgoing.iter_mut(){
        if i.0 == to {
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
