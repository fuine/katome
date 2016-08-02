use ::data::types::{Graph, VertexId, K_SIZE};
use ::data::read_slice::ReadSlice;
// use ::data::edges::{Edges, Edge};
// use asm::assembler::{SEQUENCES};
// use std::collections::hash_map::Entry;
use std::iter;
use std::slice;
use ::petgraph::graph::{Node, NodeIndex, EdgeIndex};
use ::petgraph::EdgeDirection;

pub struct Pruner<'a> {
    graph: &'a mut Graph,
}

enum VertexType<T> {
    Input(T),
    Output(T),
}

struct Externals<'a> {
    iter: iter::Enumerate<slice::Iter<'a, Node<ReadSlice, VertexId>>>,
}

impl<'a> Externals<'a> {
    fn new(iter_: iter::Enumerate<slice::Iter<'a, Node<ReadSlice, VertexId>>>) -> Externals {
        Externals {
            iter: iter_,
        }
    }
}

impl<'a> Iterator for Externals<'a> {
    type Item = VertexType<NodeIndex<VertexId>>;
    fn next(&mut self) -> Option<VertexType<NodeIndex<VertexId>>> {
        loop {
            match self.iter.next() {
                None => return None,
                Some((index, node)) => {
                    if node.next_edge(EdgeDirection::Incoming) == EdgeIndex::end() {
                        return Some(VertexType::Input(NodeIndex::new(index)));
                    }
                    else if node.next_edge(EdgeDirection::Outgoing) == EdgeIndex::end() {
                        return Some(VertexType::Output(NodeIndex::new(index)));
                    }
                    else {
                        continue
                    }
                },
            }
        }
    }
}

impl<'a> Pruner<'a> {
    /// Constructor for Pruner.
    pub fn new(graph_: &'a mut Graph) -> Pruner {
        Pruner {
            graph: graph_,
        }
    }

    /// Remove all input and output dead paths
    pub fn prune_graph(&mut self) {
        info!("Starting graph pruning");
        loop {
            debug!("Detected {} input/output vertices", Externals::new(self.graph.raw_nodes().iter().enumerate()).count());
            let mut to_remove: Vec<EdgeIndex<VertexId>> = vec![];
            // analyze found input/output vertices
            for v in Externals::new(self.graph.raw_nodes().iter().enumerate()) {
                // sort into output and input paths
                match v {
                    VertexType::Input(v_) => {
                        // decide whether or not vertex is in the dead path
                        if let Some(dead_path) = check_dead_path(self.graph, v_, EdgeDirection::Incoming, EdgeDirection::Outgoing) {
                            to_remove.extend(dead_path);
                        }
                    }
                    VertexType::Output(v_) => {
                        // decide whether or not vertex is in the dead path
                        if let Some(dead_path) = check_dead_path(self.graph, v_, EdgeDirection::Outgoing, EdgeDirection::Incoming) {
                            to_remove.extend(dead_path);
                        }
                    }
                }
            }
            // if there are no dead paths left pruning is done
            if to_remove.is_empty() {
                info!("Graph is pruned");
                return;
            }
            self.number_of_single_vertices();
            self.remove_paths(to_remove.as_slice());
            self.number_of_single_vertices();
        }
    }

    /// Remove dead input path.
    fn remove_paths(&mut self,  to_remove: &[EdgeIndex<VertexId>]) {
        debug!("Removing {} dead paths", to_remove.len());
        for e in to_remove.iter() {
            self.graph.remove_edge(*e);
                //.expect("Remove called on value which doesn't exist!");
        }
        remove_single_vertices(&mut self.graph);
    }

    fn number_of_single_vertices(&self) {
        //FIXME
        // self.graph.node_indices().map(|n| { if self.graph.neighbors_undirected(n).count() == 0 { println!("WHy"); } });
    }
}

pub fn remove_single_vertices(graph: &mut Graph) {
    graph.retain_nodes(|ref g, n| {
        if let None = g.neighbors_undirected(n).next() {
            false
        }
        else {
            true
        }
    });
}

/* /// Check if vertex initializes a dead input path.
fn check_dead_path(graph: &Graph, vertex: NodeIndex<VertexId>, first_direction: EdgeDirection, second_direction: EdgeDirection) -> Option<Vec<EdgeIndex<VertexId>>> {
    let mut output_vec = vec![];
    let mut current_vertex = vertex;
    let mut cnt = 0;
    loop {
        // this line lets us check outgoing once, without the need to iterate twice
        let next_edge = graph.first_edge(current_vertex, second_direction);
        // if vertex has more than 1 input edge
        if let Some(_) = graph.neighbors_directed(current_vertex, first_direction).nth(2) {
            return Some(output_vec);
        }
        // take first outgoing vertex
        if let Some(e) = next_edge {
            if cnt >= 2 * K_SIZE {
                // this path is not dead
                return None;
            }
            cnt += 1;
            // add vertex to path
            output_vec.push(e);
            // move to the next vertex in path
            current_vertex = graph.edge_endpoints(e).expect("Edge disappeared between lookups").1;
        }
        // if vertex has no outgoing edges
        else {
            //FIXME
            // output_vec.push(current_vertex.clone());
            return Some(output_vec);
        }
    }
} */
/// Check if vertex initializes a dead input path.
fn check_dead_path(graph: &Graph, vertex: NodeIndex<VertexId>, first_direction: EdgeDirection, second_direction: EdgeDirection) -> Option<Vec<EdgeIndex<VertexId>>> {
    let mut output_vec = vec![];
    let mut current_vertex = vertex;
    let mut cnt = 0;
    loop {
        cnt += 1;
        if cnt >= 2 * K_SIZE {
            // this path is not dead
            return None;
        }
        // this line lets us check outgoing once, without the need to iterate twice
        let next_edge = graph.first_edge(current_vertex, second_direction);
        if let Some(e) = next_edge {
            // add vertex to path
            output_vec.push(e);
            // move to the next vertex in path
            current_vertex = graph.edge_endpoints(e).expect("Edge disappeared between lookups").1;
        }
        // if vertex has no outgoing edges
        else {
            return Some(output_vec);
        }
        if let Some(_) = graph.neighbors_directed(current_vertex, first_direction).nth(2) {
            return Some(output_vec);
        }
    }
}
/*
 /// Check if vertex initializes a dead input path.
fn check_input_dead_path(graph: &Graph, vertex: &VertexId) -> Option<Vec<ReadSlice>> {
    let mut input_path = vec![];
    let mut current_vertex: ReadSlice = RS!(*vertex);
    let mut cnt = 0;
    loop {
        cnt += 1;
        if cnt >= 2 * K_SIZE {
            // this path is not dead
            return None;
        }
        // get Edges for the current vertex
        let current_value = graph.get(&current_vertex)
            .expect("Vertex doesn't exist even though someone has it in outgoing!");
        if current_value.in_num > 1 {
            return Some(input_path);
        }
        input_path.push(current_vertex.clone());
        // if vertex has no outgoing edges or has more than 1 input edge
        if current_value.outgoing.is_empty() {
            return Some(input_path);
        }
        // move to the next vertex in path
        current_vertex = RS!(current_value.outgoing[0].0);
    }
}
*/
/*
/// Check if vertex initializes a dead input path.
fn check_output_dead_path(graph: &mut Graph, vertex: &VertexId) -> Option<Vec<ReadSlice>> {
    let mut output_vec = vec![];
    let mut current_vertex: ReadSlice = RS!(*vertex);
    let mut cnt = 0;
    loop {
        // if vertex has more than 1 outgoing edges
        if let Some(_) = graph.neighbors_directed(current_vertex, EdgeDirection::Outgoing).nth(2) {
            return Some(output_vec);
        }
        // if vertex has no input edges
        else if current_value.in_num == 0 {
            output_vec.push(current_vertex.clone());
            return Some(output_vec);
        }
        else {
            if cnt >= 2 * K_SIZE {
                // this path is not dead
                return None;
            }
            cnt += 1;
            // add vertex to the path
            output_vec.push(current_vertex.clone());
            // backtrack through the path to the previous vertex
            current_vertex = get_input_vertices(graph, &current_vertex, true)
                .pop()
                .expect(format!("Didn't find predecessor despite having {} in_nums for slice: {}", current_value.in_num, current_vertex.name()).as_str());
        }
    }
} */
