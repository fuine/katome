use ::data::types::{Graph, VertexId, K_SIZE, EdgeWeight};
use ::data::read_slice::ReadSlice;
use std::iter;
use std::slice;
use ::petgraph::graph::{Node, NodeIndex, EdgeIndex};
use ::petgraph::EdgeDirection;


/// Various algorithms for graph pruning - removing unnecessary vertices/edges

enum VertexType<T> {
    Input(T),
    Output(T),
}

/// Iterator yielding vertices which either have no incoming or outgoing edges.
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

/// Remove all input and output dead paths
pub fn remove_dead_paths(graph: &mut Graph) {
    info!("Starting graph pruning");
    loop {
        debug!("Detected {} input/output vertices", Externals::new(graph.raw_nodes().iter().enumerate()).count());
        let mut to_remove: Vec<EdgeIndex<VertexId>> = vec![];
        // analyze found input/output vertices
        for v in Externals::new(graph.raw_nodes().iter().enumerate()) {
            // sort into output and input paths
            match v {
                VertexType::Input(v_) => {
                    // decide whether or not vertex is in the dead path
                    if let Some(dead_path) = check_dead_path(graph, v_, EdgeDirection::Incoming, EdgeDirection::Outgoing) {
                        to_remove.extend(dead_path);
                    }
                }
                VertexType::Output(v_) => {
                    // decide whether or not vertex is in the dead path
                    if let Some(dead_path) = check_dead_path(graph, v_, EdgeDirection::Outgoing, EdgeDirection::Incoming) {
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
        // reverse sort edge indices such that removal won't cause any troubles with swapped
        // edge indices (see petgraph's explanation of remove_edge())
        to_remove.sort_by(|a, b| b.cmp(a));
        remove_paths(graph, to_remove.as_slice());
    }
}

/// Remove dead input path.
fn remove_paths(graph: &mut Graph,  to_remove: &[EdgeIndex<VertexId>]) {
    debug!("Removing {} dead paths", to_remove.len());
    for e in to_remove.iter() {
        graph.remove_edge(*e);
    }
    remove_single_vertices(graph);
}

/// Remove vertives without any edges.
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

/// Remove edges with weight below threshold.
pub fn remove_weak_edges(graph: &mut Graph, threshold: EdgeWeight) {
    graph.retain_edges(|g, e| {
        !(*g.edge_weight(e).unwrap() < threshold)
    });
    remove_single_vertices(graph);
}

/*
pub fn remove_not_connected_vertices(graph: &mut Graph) {
    let keys_to_remove: Vec<ReadSlice> = graph.iter()
        .filter(|&(_, ref val)| val.outgoing.is_empty() && val.in_num == 0)
        .map(|(key, _)| key.clone())
        .collect();
    for key in keys_to_remove {
        graph.remove(&key);
    }
} */
