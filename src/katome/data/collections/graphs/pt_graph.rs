//! `petgraph` based `Graph`.
use ::petgraph;

use algorithms::builder::Build;
use asm::SEQUENCES;
use data::collections::graphs::Graph;
use data::primitives::{K_SIZE, K1_SIZE, EdgeWeight, Idx};
use data::read_slice::ReadSlice;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::Entry;
use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
pub use petgraph::dot::Dot;


/// Type denoting index of edge.
pub type EdgeIndex = petgraph::graph::EdgeIndex<Idx>;
/// Type denoting index of node.
pub type NodeIndex = petgraph::graph::NodeIndex<Idx>;
/// `Node` type in `PtGraph`.
pub type Node = petgraph::graph::Node<ReadSlice, Idx>;

/// `petgraph` based `Graph`.
pub type PtGraph = petgraph::Graph<ReadSlice, EdgeWeight, petgraph::Directed, Idx>;

/// Serialize graph into .dot file.
pub fn write_to_dot(graph: &PtGraph, path_: &str) {
    let path = Path::new(path_);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
        Ok(file) => file,
    };
    match write!(file, "{}", Dot::new(graph)) {
        Err(why) => {
            panic!("couldn't write to {}: {}", display,
                   why.description())
        },
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

impl Graph for PtGraph {
    type NodeIdentifier = NodeIndex;
    type AmbiguousNodes = HashSet<NodeIndex>;
    fn get_ambiguous_nodes(&self) -> Self::AmbiguousNodes {
        self.node_indices()
            .filter(|n| {
                let in_degree = self.in_degree(n);
                let out_degree = self.out_degree(n);
                (in_degree > 1 || out_degree > 1) || (in_degree == 0 && out_degree >= 1)
            })
            .collect::<Self::AmbiguousNodes>()
    }

    fn out_degree(&self, node: &Self::NodeIdentifier) -> usize {
        self.neighbors_directed(*node, petgraph::EdgeDirection::Outgoing)
            .count()
    }

    fn in_degree(&self, node: &Self::NodeIdentifier) -> usize {
        self.neighbors_directed(*node, petgraph::EdgeDirection::Incoming)
            .count()
    }
}

type ReadsToNodes = HashMap<ReadSlice, NodeIndex>;

// Graph builder which stores information about already seen vertices
#[derive(Default)]
struct PtGraphBuilder {
    graph: PtGraph,
    reads_to_nodes: ReadsToNodes,
}

impl Build for PtGraphBuilder {
    fn add_read(&mut self, read: &[u8]) {
        assert!(read.len() as Idx >= K_SIZE, "Read is too short!");
        let mut ins_counter: Idx = 0;
        let mut index_counter = SEQUENCES.read().len() as Idx;
        let mut current_node: NodeIndex;
        let mut previous_node: NodeIndex = NodeIndex::new(0);
        let mut offset;
        let mut insert = false;
        // let mut prev_val_old: *mut Edges = 0 as *mut Edges;
        for (cnt, window) in read.windows(K1_SIZE as usize).enumerate(){
            let from_tmp = {
                let mut s = SEQUENCES.write();
                offset = s.len();
                s.extend_from_slice(window);
                ReadSlice::new(offset as Idx)
            };
            current_node = { // get a proper key to the hashmap
                match self.reads_to_nodes.entry(from_tmp) {
                    Entry::Occupied(oe) => {
                        SEQUENCES.write().truncate(offset);
                        if ins_counter > 0 {
                            ins_counter += 1;
                        }
                        *oe.get()
                    }
                    Entry::Vacant(_) => { // we cant use that VE because it is keyed with a temporary value
                        SEQUENCES.write().truncate(offset);
                        // push to vector
                        if ins_counter == 0 {
                            // append window to vector
                            SEQUENCES.write().extend_from_slice(window);
                        }
                        else if ins_counter > K1_SIZE {
                            // append window to vector
                            SEQUENCES.write().extend_from_slice(window);
                            index_counter += K1_SIZE;
                        }
                        else {
                            // append only ins_counter last bytes of window
                            SEQUENCES.write().extend_from_slice(&window[(K1_SIZE - ins_counter ) as usize ..]);
                            index_counter += ins_counter;
                        }
                        ins_counter = 1;
                        insert = true;
                        self.graph.add_node(ReadSlice::new(index_counter))
                    }
                }
            };
            if insert {
                self.reads_to_nodes.insert(ReadSlice::new(index_counter), current_node);
                insert = false;
            }
            if cnt > 0 { // insert current sequence as a member of the previous
                update_edge(&mut self.graph, previous_node, current_node);
            }
            previous_node = current_node;
        }
    }
}

impl Build for PtGraph {
    fn create<P: AsRef<Path>>(path: P) -> (Self, usize) where Self: Sized {
        let (builder, number_of_read_bytes) = PtGraphBuilder::create(path);
        (builder.graph, number_of_read_bytes)
    }

    #[allow(unused_variables)]
    fn add_read(&mut self, read: &[u8]) {
        unimplemented!();
    }
}

fn update_edge(graph: &mut PtGraph, a: NodeIndex, b: NodeIndex) {
    if let Some(ix) = graph.find_edge(a, b) {
        if let Some(ed) = graph.edge_weight_mut(ix) {
            *ed += 1;
            return;
        }
    }
    graph.add_edge(a, b, 1);
}
