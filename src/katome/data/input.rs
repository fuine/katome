// input.rs
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::slice;
// use data::edges::{Edges};
use std::collections::HashMap;
use std::collections::hash_map::{Entry};
use data::read_slice::ReadSlice;
use data::types::{Graph, MyHasher,
                  VertexId, K_SIZE};
use asm::assembler::{SEQUENCES};
use std::io::BufReader;
// use ::pbr::{ProgressBar};
use ::petgraph::graph::NodeIndex;

type ReadsToNodes = HashMap<ReadSlice, NodeIndex<VertexId>, MyHasher>;


// creates graph
pub fn read_sequences(path: String, graph: &mut Graph,
                      saved: &mut VertexId, total: &mut usize, number_of_reads: &mut usize) {
    // let line_count = count_lines(&path) / 4;
    // let chunk = line_count / 100;
    // let mut cnt = 0;
    // let mut pb = ProgressBar::new(24294983 as u64);
    // let mut pb = ProgressBar::new(100 as u64);
    // pb.format("╢▌▌░╟");
    let mut lines = match lines_from_file(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path,
                                                   Error::description(&why)),
        Ok(lines) => lines,
    };
    let mut register = vec![];
    let mut reads_to_nodes: ReadsToNodes = HashMap::default();
    loop {
        if let None = lines.next() { break }  // read line -- id
        register.clear(); // remove last line
        register = lines.next().unwrap().unwrap().into_bytes();
        *total += register.len() as VertexId;
        add_sequence_to_graph(&register, graph, &mut reads_to_nodes, saved);
        lines.next(); // read +
        lines.next(); // read quality
        *number_of_reads += 1;
    }
}

fn add_sequence_to_graph(
        read: &[u8], graph: &mut Graph, reads_to_nodes: &mut ReadsToNodes, saved: &mut VertexId) {
    assert!(read.len() as VertexId >= K_SIZE + 1, "Read is too short!");
    let mut ins_counter: VertexId = 0;
    let mut index_counter = SEQUENCES.read().unwrap().len() as VertexId;
    let mut current_node: NodeIndex<VertexId>;
    let mut previous_node: NodeIndex<VertexId> = NodeIndex::new(0);
    let mut offset;
    let mut insert = false;
    // let mut prev_val_old: *mut Edges = 0 as *mut Edges;
    for (cnt, window) in read.windows(K_SIZE as usize).enumerate(){
        let from_tmp = {
            let mut s = SEQUENCES.write().unwrap();
            offset = s.len();
            s.extend_from_slice(window);
            RS!(offset as VertexId)
        };
        current_node = { // get a proper key to the hashmap
            match reads_to_nodes.entry(from_tmp) {
                Entry::Occupied(oe) => {
                    SEQUENCES.write().unwrap().truncate(offset);
                    if ins_counter > 0 {
                        ins_counter += 1;
                    }
                    *oe.get()
                }
                Entry::Vacant(_) => { // we cant use that VE because it is keyed with a temporary value
                    SEQUENCES.write().unwrap().truncate(offset);
                    // push to vector
                    if ins_counter == 0 {
                        // append window to vector
                        SEQUENCES.write().unwrap().extend_from_slice(window);
                        *saved += K_SIZE;
                    }
                    else if ins_counter > K_SIZE {
                        // append window to vector
                        SEQUENCES.write().unwrap().extend_from_slice(window);
                        index_counter += K_SIZE;
                        *saved += K_SIZE;
                    }
                    else {
                        // append only ins_counter last bytes of window
                        SEQUENCES.write().unwrap().extend_from_slice(&window[(K_SIZE - ins_counter ) as usize ..]);
                        index_counter += ins_counter;
                        *saved += ins_counter;
                    }
                    ins_counter = 1;
                    insert = true;
                    graph.add_node(RS!(index_counter))
                }
            }
        };
        if insert {
            reads_to_nodes.insert(RS!(index_counter), current_node);
            insert = false;
        }
        if cnt > 0 { // insert current sequence as a member of the previous
            update_edge(graph, previous_node, current_node);
        }
        previous_node = current_node;
    }
}

fn update_edge(graph: &mut Graph, a: NodeIndex<VertexId>, b: NodeIndex<VertexId>) {
    if let Some(ix) = graph.find_edge(a, b) {
        if let Some(ed) = graph.edge_weight_mut(ix) {
            *ed += 1;
            return;
        }
    }
    graph.add_edge(a, b, 1);
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

pub fn as_u8_slice(v: &u8, size: usize) -> &[u8] {
    unsafe{
        slice::from_raw_parts(v, size)
    }
}
