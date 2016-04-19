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
    // let mut lines = match lines_from_file(&path) {
        // Err(why) => panic!("Couldn't open {}: {}", path,
                                                   // Error::description(&why)),
        // Ok(lines) => lines,
    // };
    // let register: VecArc = Arc::new(RefCell::new(Vec::with_capacity(100))); // registry for a single line
    // loop {
        // match lines.next() { // read line -- id
            // None => { break },
            // _ => {}
        // }
        // // TODO exit gracefully if format is wrong
        // register.borrow_mut().clear(); // remove last line
        // // XXX consider using append
        // register.borrow_mut().extend_from_slice(lines.next().unwrap().unwrap().into_bytes().as_slice());
        // add_sequence_to_graph(register.clone(), graph, reads.clone());

        // lines.next(); // read +
        // lines.next(); // read quality
        // // pb.inc();
    // }
}

pub fn add_sequence_to_graph(
        vec: VecArc, graph: &mut Graph, reads: VecArc) {
    // let iterations = vec.len() - K_SIZE;
    // // let refcount: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(vec.clone()));
    // let ins_counter = 0;
    // // let mut inserted = false;
    // let mut cnt = 0;
    // let mut first_iter = true;
    // for window in vec.windows(K_SIZE){
        // if first_iter {
            // first_iter = false;
            // let from = ReadSlice::new(vec.clone(), cnt);
            // let to   = ReadSlice::new(vec.clone(), cnt + 1);
            // let mut found = true;
            // match graph.get_mut(&from) {
                // Some(edges) => {
                    // ins_counter += 1;
                    // modify_edge(edges, to);
                // }
                // None => found = false
            // }
            // if !found { // we need to insert a new sequence and keep it's pointer valid
                // let seq_offset = reads.len();
                // if ins_counter == 0 || ins_counter >= K_SIZE {
                    // // append whole window
                // }
                // else {
                    // // append specified bytes
                // }
                // graph.insert(ReadSlice::new(ptrs.0), Edges::new(ReadSlice::new(ptrs.1)));
            // };
        // }
        // else {

        // }
        // if cnt == total_size -1 { // last iter
            // let from = ReadSlice::new(&window[0] as VertexId);
            // let mut found = true;
            // match graph.get_mut(&from) {
                // Some(edges) => {},
                // None => found = false,
            // }
            // if !found {
                // let ptrs: VertexId= match inserted {
                    // Some(ref seq) => (&(**seq)[cnt]), //unwrap ref to box and then box itself
                    // None          => {
                        // let s: ReadPtr = Box::new(vec.clone());  // sequence is on the heap now
                        // let from_: VertexId = &(*s)[cnt];
                        // inserted = Some(s);
                        // from_
                    // }
                // };
                // // let ptr: *const u8 = &(*seq)[0];
                // graph.insert(ReadSlice::new(ptrs), Edges::empty());
            // }
        // }
        // else {

        // }

        // cnt += 1;
    // }
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
