use ::data::types::{Graph, VertexId, K_SIZE};
use ::data::read_slice::ReadSlice;
use ::data::edges::{Edges, Edge};
use asm::assembler::{SEQUENCES};
use std::collections::hash_map::Entry;

pub struct Pruner<'a> {
    graph: &'a mut Graph,
}

enum VertexType<T> {
    Input(T),
    Output(T),
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
            // grab all input and output vertices
            let inputs = self.graph.iter()
                .filter(|&(_, val)| val.in_num == 0 || val.outgoing.is_empty())
                .map(|(key, val)| {
                    if val.in_num == 0 {
                        VertexType::Input(key.offset)
                    }
                    else {
                        VertexType::Output(key.offset)
                    }
                })
                // FIXME change VertexId to ReadSlice
                .collect::<Vec<VertexType<VertexId>>>();
            debug!("Detected {} input/output vertices", inputs.len());
            let mut to_remove_in: Vec<ReadSlice> = vec![];
            let mut to_remove_out: Vec<ReadSlice> = vec![];
            // analyze found input/output vertices
            for v in inputs {
                // sort into output and input paths
                match v {
                    VertexType::Input(v_) => {
                        // decide whether or not vertex is in the dead path
                        if let Some(dead_path) = check_input_dead_path(self.graph, &v_) {
                            to_remove_in.extend(dead_path);
                        }
                    }
                    VertexType::Output(v_) => {
                        // decide whether or not vertex is in the dead path
                        if let Some(dead_path) = check_output_dead_path(self.graph, &v_) {
                            to_remove_out.extend(dead_path);
                        }
                    }
                }
            }
            // if there are no dead paths left pruning is done
            if to_remove_in.is_empty() && to_remove_out.is_empty() {
                info!("Graph is pruned");
                return;
            }
            if !to_remove_in.is_empty() {
                // remove dead input paths
                self.remove_in_path(to_remove_in.as_slice());
            }
            if !to_remove_out.is_empty() {
                // remove dead output paths
                self.remove_out_path(to_remove_out.as_slice());
            }
        }
    }

    /// Remove dead input path.
    fn remove_in_path(&mut self,  to_remove: &[ReadSlice]) {
        debug!("Removing {} dead input paths", to_remove.len());
        for v in to_remove.iter() {
            // Remove the vertex and catch it's Edges
            let removed: Edges = self.graph.remove(v)
                .expect("Remove called on value which doesn't exist!");
            // decrement input edges counter for each vertex in the outgoing set
            for o in removed.outgoing.iter() {
                self.graph.get_mut(&RS!(o.0)).unwrap().in_num -= 1;
            }
        }
    }

    /// Remove dead outgoing path.
    fn remove_out_path(&mut self, to_remove: &[ReadSlice]) {
        debug!("Removing {} dead output paths", to_remove.len());
        for v in to_remove.iter() {
            // get all edges that are incoming to the vertex to be removed
            let outgoing = get_input_vertices(self.graph, v, false);
            // for each edge remove it from outgoing set in the starting vertex
            for o in outgoing {
                // find outgoing set for the starting vertex
                let edges: &mut Edges = self.graph.get_mut(&o)
                    .expect("Vertex from get_input_vertices doesn't exist!");
                let mut out_ = Vec::new();
                out_.extend_from_slice(&edges.outgoing);
                // filter out the vertex we are about to remove
                let out: Vec<Edge> = out_.into_iter().filter(|&x| x.0 != v.offset).collect();
                // save filtered outgoing set
                edges.outgoing = out.into_boxed_slice();
            }
            // finally remove the vertex itself
            self.graph.remove(v);
        }
    }

}
/// Utility function which gets us every possible incoming edge.
/// because of memory savings we do not hold an array of incoming edges,
/// instead we will exploit the idea behind sequencing genome, namely
/// common bytes for each sequence.
/// WARNING: this may or may not be optimal if we follow the fasta standard
/// but should be sufficiently faster for just 5 characters we use at the moment
fn get_input_vertices(graph: &mut Graph, vertex: &ReadSlice, one_vertex: bool) -> Vec<ReadSlice> {
    // FIXME ugly hack to work with static vector
    // create register
    let mut output: Vec<ReadSlice> = vec![];
    // let register: VecArc = Arc::new(RefCell::new(Vec::with_capacity(K_SIZE))); // register for a single line
    let offset;
    {
        let mut vec = vec![];
        // copy current sequence to register
        vec.extend(vertex.get_slice());
        // shift the register one character to the right
        vec.truncate(K_SIZE - 1);
        vec.insert(0, 0);
        let mut s = SEQUENCES.write().unwrap();
        offset = s.len() as VertexId;
        s.extend_from_slice(vec.as_slice());
    }
    // try to bruteforce by inserting all possible characters: ACTGN
    for chr in &['A', 'C', 'T', 'G', 'N'] {
    // for chr in 65..90 {
        SEQUENCES.write().unwrap()[0] = *chr as u8;
        // dummy read slice used to check if we can find it in the graph
        // let tmp_rs = ReadSlice::new(register.clone(), 0);
        let tmp_rs = RS!(offset);
        if let Entry::Occupied(e) = graph.entry(tmp_rs) {
            // if we got any hits check if our vertex is in the outgoing
            if let Some(_) = e.get().outgoing.iter().find(|&x| x.0 == vertex.offset) {
                // if so, then add to output array
                // trace!("Found input vertex: {}", e.key().name());
                output.push(e.key().clone());
                if one_vertex {
                    break;
                }
            }
        }
    }
    SEQUENCES.write().unwrap().truncate(offset);
    output
}

/// Check if vertex initializes a dead input path.
fn check_input_dead_path(graph: &Graph, vertex: &VertexId) -> Option<Vec<ReadSlice>> {
    let mut output_vec = vec![];
    let mut current_vertex: ReadSlice = RS!(*vertex);
    let mut cnt = 0;
    loop {
        // get Edges for the current vertex
        let current_value = graph.get(&current_vertex)
            .expect("Vertex doesn't exist even though someone has it in outgoing!");
        // if vertex has more than 1 input edge
        if current_value.in_num > 1 {
            return Some(output_vec);
        }
        // if vertex has no outgoing edges
        else if current_value.outgoing.is_empty() {
            output_vec.push(current_vertex.clone());
            return Some(output_vec);
        }
        else {
            if cnt >= 2 * K_SIZE {
                // this path is not dead
                return None;
            }
            cnt += 1;
            // add vertex to path
            output_vec.push(current_vertex.clone());
            // move to the next vertex in path
            current_vertex = RS!(current_value.outgoing[0].0);
        }
    }
}

/// Check if vertex initializes a dead input path.
fn check_output_dead_path(graph: &mut Graph, vertex: &VertexId) -> Option<Vec<ReadSlice>> {
    let mut output_vec = vec![];
    let mut current_vertex: ReadSlice = RS!(*vertex);
    let mut cnt = 0;
    loop {
        // get Edges for the current vertex
        let current_value  = graph.get(&current_vertex)
            .expect("Couldn't find vertex which supposedly is in the graph!")
            .clone();
        // if vertex has more than 1 outgoing edges
        if current_value.outgoing.len() > 1 {
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
}
