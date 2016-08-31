//! Graph's Intermediate Representation
use alloc::heap::deallocate;
use std::mem;
use asm::assembler::SEQUENCES;
use data::edges::Edges;
use data::read_slice::ReadSlice;
use data::graph::{Graph, K_SIZE, Idx, NodeIndex};
use data::vertex::{Vertex};

use std::collections::HashSet as HS;
use std::error::Error;
use std::fs::File;
use std::hash::BuildHasherDefault;
use std::io::BufReader;
use std::io::prelude::*;
use std::io;
use std::path::Path;

extern crate metrohash;
use self::metrohash::MetroHash;
// use ::pbr::{ProgressBar};

/// Graph's Intermediate Representation (GIR) is used as a middle step during creation of the
/// graph. It deals with data of unknown size better, because it uses only one underlying
/// collection, namely hashmap, as opposed to petgraph's two vectors and additional collection to
/// track already seen sequences.
pub type GIR = HS<Box<Vertex>, BuildHasherDefault<MetroHash>>;

/// Create `GIR` from the supplied fastaq file.
pub fn create_gir(path: String) -> (GIR, usize) {
    /*
    let line_count = count_lines(&path) / 4;
    let chunk = line_count / 100;
    let mut cnt = 0;
    let mut pb = ProgressBar::new(24294983 as u64);
    let mut pb = ProgressBar::new(100 as u64);
    pb.format("╢▌▌░╟");
    */
    let mut total = 0usize;
    let mut gir: GIR = GIR::default();
    let mut lines = match lines_from_file(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path, Error::description(&why)),
        Ok(lines) => lines,
    };
    let mut register = vec![];
    info!("Starting to build GIR");
    loop {
        if let None = lines.next() { break; }  // read line -- id
        // TODO exit gracefully if format is wrong
        register.clear(); // remove last line
        // XXX consider using append
        register = lines.next().unwrap().unwrap().into_bytes();
        total += register.len() as Idx;
        add_read_to_gir(&register, &mut gir);
        lines.next(); // read +
        lines.next(); // read quality
        /*
        cnt += 1;
        if cnt >= chunk {
            cnt = 0;
            pb.inc();
        }
        */
    }
    info!("GIR built");
    (gir, total)
}

/// Add new reads to `GIR`, modify weights of existing edges.
fn add_read_to_gir(read: &[u8], gir: &mut GIR) {
    assert!(read.len() as Idx >= K_SIZE + 1, "Read is too short!");
    let mut ins_counter: Idx = 0;
    let mut current: Box<Vertex>;
    let mut previous_node = Box::new(Vertex::new(RS!(0), Edges::default()));
    let mut offset;
    let mut idx = gir.len();
    let mut current_idx;
    let mut insert = false;
    for (cnt, window) in read.windows(K_SIZE as usize).enumerate() {
        let rs = {
            let mut s = unwrap!(SEQUENCES.write(), "Global sequences poisoned :(");
            offset = s.len();
            // append new data to the global vector of sequences
            if ins_counter == 0 || ins_counter > K_SIZE {
                // append window to vector
                s.extend_from_slice(window);
                RS!(offset as Idx)
            }
            else {
                // append only ins_counter last bytes of window
                s.extend_from_slice(&window[(K_SIZE - ins_counter) as usize..]);
                RS!(offset - (K_SIZE - ins_counter) as Idx)
            }
        };
        current = Box::new(Vertex::new(rs, Edges::empty(idx)));
        if let Some(v) = gir.get(&current) {
            // sequence already exists, we should remove redundant bytes from
            // SEQUENCES and update counters
            if ins_counter > 0 {
                ins_counter += 1;
            }
            unwrap!(SEQUENCES.write()).truncate(offset);
            current_idx = v.edges.idx;
            current = v.clone();
        }
        else {
            insert = true;
            ins_counter = 1;
            current_idx = idx;
            idx += 1;
        }
        if insert {
            gir.insert(current.clone());
            insert = false;
        }
        if cnt > 0 {
            create_or_modify_edge(&mut previous_node.edges, current_idx);
            gir.replace(previous_node);
        }
        previous_node = current;
    }
}

/// Create edge if it previously haven't existed, otherwise increase it's weight.
fn create_or_modify_edge(edges: &mut Edges, to: Idx) {
    for i in edges.outgoing.iter_mut() {
        if i.0 == to {
            i.1 += 1;
            return;
        }
    }
    let mut out_ = Vec::new();
    out_.extend_from_slice(&edges.outgoing);
    out_.push((to, 1));
    edges.outgoing = out_.into_boxed_slice();
}

/// Count lines in the supplied file.
#[allow(dead_code)]
fn count_lines(filename: &str) -> usize {
    let file = File::open(filename).expect("I couldn't open that file, sorry :(");

    let reader = BufReader::new(file);

    reader.split(b'\n').count()
}

fn lines_from_file<P>(filename: P) -> Result<io::Lines<io::BufReader<File>>, io::Error>
    where P: AsRef<Path> {
    let file = try!(File::open(filename));
    Ok(io::BufReader::new(file).lines())
}

/// Convert GIR to petgraph's Graph implementation. At this stage assembler loses information about
/// already seen sequences (in the sense of reasonable, efficient and repeatable check - one can
/// always use iterator with find, which pessimistically yields complexity of O(n), as opposed to
/// O(1) for hashmap).
pub fn gir_to_graph(gir: GIR) -> Graph {
    let mut graph = Graph::default();
    let size = mem::size_of::<Vertex>();
    let align = mem::align_of::<Vertex>();
    for vertex in gir.into_iter() {
        let source = NodeIndex::new(vertex.edges.idx);
        while source.index() >= graph.node_count() {
            graph.add_node(ReadSlice::default());
        }
        for edge in vertex.edges.outgoing.into_iter() {
            while edge.0 >= graph.node_count() {
                graph.add_node(ReadSlice::default());
            }
            graph.add_edge(source, NodeIndex::new(edge.0), edge.1);
        }
        *unwrap!(graph.node_weight_mut(source)) = vertex.rs.clone();

        // deallocate box such that it does not occupy memory
        let raw = Box::into_raw(vertex);
        unsafe { deallocate(raw as *mut _, size, align) };
    }
    graph
}
