// Graph intermediate representation
// extern crate twox_hash;
// use self::twox_hash::XxHash;
use std::hash::BuildHasherDefault;
extern crate metrohash;
// use std::collections::hash_state::DefaultState;
use self::metrohash::MetroHash;
use std::collections::HashMap as HM;
use data::edges::Edges;
use data::read_slice::ReadSlice;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::collections::hash_map::{Entry};
use data::types::{VertexId, K_SIZE, Graph};
use asm::assembler::{SEQUENCES};
use std::io::BufReader;
use ::petgraph::graph::{NodeIndex};
// use ::pbr::{ProgressBar};

// pub type MyHasher = BuildHasherDefault<XxHash>;
pub type MyHasher = BuildHasherDefault<MetroHash>;
/// Graph's Intermediate Representation (GIR) is used as a middle step during creation of the
/// graph. It deals with data of unknown size better, because it uses only one underlying
/// collection, namely hashmap, as opposed to petgraph's two vectors and additional collection to
/// track already seen sequences.
pub type GIR = HM<ReadSlice, Edges, MyHasher>;

pub fn create_gir(path: String) -> (GIR, usize, usize) {
    /* let line_count = count_lines(&path) / 4;
    let chunk = line_count / 100;
    let mut cnt = 0;
    let mut pb = ProgressBar::new(24294983 as u64);
    let mut pb = ProgressBar::new(100 as u64);
    pb.format("╢▌▌░╟"); */
    let mut saved = 0usize;
    let mut total = 0usize;
    let mut gir: GIR = GIR::default();
    let mut lines = match lines_from_file(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path,
                                                   Error::description(&why)),
        Ok(lines) => lines,
    };
    let mut register = vec![];
    info!("Starting to build GIR");
    loop {
        if let None = lines.next() { break }  // read line -- id
        // TODO exit gracefully if format is wrong
        register.clear(); // remove last line
        // XXX consider using append
        register = lines.next().unwrap().unwrap().into_bytes();
        total += register.len() as VertexId;
        saved += add_sequence_to_gir(&register, &mut gir);
        lines.next(); // read +
        lines.next(); // read quality
        /* cnt += 1;
        if cnt >= chunk {
            cnt = 0;
            pb.inc();
        } */
    }
    info!("GIR built");
    (gir, saved, total)
}

pub fn add_sequence_to_gir(
        read: &[u8], gir: &mut GIR) -> usize {
    assert!(read.len() as VertexId >= K_SIZE + 1, "Read is too short!");
    let mut ins_counter: VertexId = 0;
    let mut index_counter = SEQUENCES.read().unwrap().len() as VertexId;
    let mut current: ReadSlice;
    let mut insert = false;
    let mut previous_node: ReadSlice = RS!(0);
    let mut offset;
    let mut idx = gir.len();
    let mut current_idx;
    let mut saved = 0;
    for (cnt, window) in read.windows(K_SIZE as usize).enumerate(){
        let from_tmp = {
            let mut s = SEQUENCES.write().unwrap();
            offset = s.len();
            s.extend_from_slice(window);
            RS!(offset as VertexId)
        };
        current = { // get a proper key to the hashmap
            match gir.entry(from_tmp) {
                Entry::Occupied(oe) => {
                    SEQUENCES.write().unwrap().truncate(offset);
                    if ins_counter > 0 {
                        ins_counter += 1;
                    }
                    current_idx = oe.get().idx;
                    oe.key().clone()
                }
                Entry::Vacant(_) => { // we cant use that VE because it is keyed with a temporary value
                    SEQUENCES.write().unwrap().truncate(offset);
                    // push to vector
                    if ins_counter == 0 {
                        // append window to vector
                        SEQUENCES.write().unwrap().extend_from_slice(window);
                        saved += K_SIZE;
                    }
                    else if ins_counter > K_SIZE {
                        // append window to vector
                        SEQUENCES.write().unwrap().extend_from_slice(window);
                        index_counter += K_SIZE;
                        saved += K_SIZE;
                    }
                    else {
                        // append only ins_counter last bytes of window
                        SEQUENCES.write().unwrap().extend_from_slice(&window[(K_SIZE - ins_counter ) as usize ..]);
                        index_counter += ins_counter;
                        saved += ins_counter;
                    }
                    ins_counter = 1;
                    current_idx = idx;
                    insert = true;
                    RS!(index_counter)
                }
            }
        };
        if cnt > 0 { // insert current sequence as a member of the previous
            let e: &mut Edges = gir.get_mut(&previous_node).unwrap();
            modify_edge(e, current_idx);
        }
        if insert {
            gir.entry(current.clone()).or_insert_with(|| Edges::empty(current_idx));
            idx += 1;
            insert = false;
        }
        previous_node = current;
    }
    saved
}

fn modify_edge(edges: &mut Edges, to: VertexId) {
    for i in edges.outgoing.iter_mut(){
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
    info!("Starting conversion from GIR to graph");
    // get rid of hashes -- we don't need them anymore
    let mut vec = gir.into_iter().collect::<Vec<(ReadSlice, Edges)>>();
    // sort this vector according to indicees of nodes, guaranteeing proper node creation (node
    // indices are created just like we created ours, but iterator over hashmap likely changed the
    // ordering).
    vec.sort_by(|a, b| a.1.idx.cmp(&b.1.idx));
    // create separate representations of nodes and edges
    let (nodes, edges): (Vec<ReadSlice>, Vec<Edges>) = vec.into_iter().unzip();
    let mut graph = Graph::default();
    // digest nodes and move them into the Graph
    for (cnt, node) in nodes.into_iter().enumerate() {
        let tmp = graph.add_node(node).index();
        assert_eq!(tmp, cnt);
    }
    for edges_ in edges.into_iter() {
        let idx = edges_.idx;
        for edge in edges_.outgoing.into_iter() {
            graph.add_edge(NodeIndex::new(idx), NodeIndex::new(edge.0), edge.1);
        }
    }
    info!("Conversion ended!");
    graph
}
