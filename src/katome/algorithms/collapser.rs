use ::data::types::{Graph, VertexId};
use ::data::read_slice::ReadSlice;
// use asm::assembler::{SEQUENCES};
use ::algorithms::hardener::remove_not_connected_vertices;
use std::fs::File;
use std::error::Error;
use std::io::Write;

pub fn collapse(graph: &mut Graph, filename: String) {
    // open file to writing
    let mut fh = match File::create(&filename) {
        Err(why) => {
            panic!("Couldn't create output file {}: {}",
                   filename,
                   why.description())
        }
        Ok(file) => file,
    };
    info!("Starting graph collapse");
    // find input indices
    while !graph.is_empty() {
        let starting_vertices = get_starting_vertices(graph);
        if starting_vertices.is_empty() && !graph.is_empty() {
            // cycle
            info!("Found a cycle");
            // randomly choose one vertex from the graph
            let vertex = find_weakest_edge(graph);
            // let vertex = graph.keys().next().unwrap().clone();
            let contig = create_contig(&vertex, graph, true);
            write_contig(contig, &mut fh);
        }
        // for each input index
        for starting_vertex in starting_vertices {
            let contig = create_contig(&starting_vertex, graph, false);
            write_contig(contig, &mut fh);
        }
    }
}

fn write_contig(mut contig: String, fh: &mut File) {
    contig.push('\n');
    match fh.write_all(contig.as_bytes()) {
        Ok(_) => (),
        Err(e) => error!("Couldn't write contig {} to file: {}", contig, e),
    }
}

fn create_contig(starting_vertex: &ReadSlice, graph: &mut Graph, cycle: bool) -> String {
    let mut contig: String;
    // if first -> write whole sequence
    let mut next_edge: VertexId;
    let mut vertex: ReadSlice;
    let mut remove_edge = false;
    {
        let mut starting_edges = graph.get_mut(starting_vertex)
            .expect("Couldn't find vertex which is supposed to be in the graph.");
        if cycle && starting_edges.in_num != 1 {
            panic!("Cycle starting vertex has weird number of in_num: {}",
                   starting_edges.in_num);
        }
        contig = starting_vertex.name();
        if starting_edges.outgoing.is_empty() {
            panic!("Empty outgoing on starting edge with {} in_num!",
                   starting_edges.in_num);
        }
        next_edge = starting_edges.outgoing[0].0;
        vertex = RS!(next_edge);
        // delete this edge
        starting_edges.outgoing[0].1 -= 1;
        if starting_edges.outgoing[0].1 == 0 {
            // end vertex
            // remove empty outgoing edges
            remove_edge = true;
            starting_edges.remove_edge(0);
        }
    }
    loop {
        let mut edges = graph.get_mut(&vertex)
            .expect("Couldn't find vertex which is supposed to be in the graph.");
        if remove_edge && cycle && vertex == *starting_vertex {
            // we arrived at the beggining - time to end
            edges.in_num -= 1;
            trace!("End of cycle!");
            break;
        }
        contig.push(vertex.last_char());
        if !((edges.outgoing.len() == 1 && edges.in_num == 0) ||
             (edges.outgoing.len() == 1 && edges.in_num == 1)) ||
           edges.outgoing.is_empty() {
            if remove_edge {
                edges.in_num -= 1;
            }
            break;
        }
        else if remove_edge {
            edges.in_num -= 1;
            remove_edge = false;
        }
        // remove edge
        edges.outgoing[0].1 -= 1;
        next_edge = edges.outgoing[0].0;
        if edges.outgoing[0].1 == 0 {
            // end vertex
            // remove empty outgoing
            edges.remove_edge(0);
            remove_edge = true;
        }
        vertex = RS!(next_edge);
    }
    // remove dead indices
    remove_not_connected_vertices(graph);
    contig
}

fn get_starting_vertices(graph: &Graph) -> Vec<ReadSlice> {
    graph.iter()
        .filter(|&(_, val)| val.in_num == 0)
        .map(|(key, _)| key.clone())
        .collect::<Vec<ReadSlice>>()
}

fn find_weakest_edge(graph: &Graph) -> ReadSlice {
    // FIXME
    // let weakest_id = graph.values()
    // .filter(|&val| !val.outgoing.is_empty())
    // .fold((0, 0), |weakest, val| {
    // for
    // });
    // RS!(vec, weakest_id)
    graph.iter()
        .skip_while(|&(_, val)| val.outgoing.is_empty())
        .map(|(key, _)| key.clone())
        .next()
        .expect("There are no edges in the graph, despite in_num being > 1 for some vertices!")
}

#[cfg(test)]
mod tests {
    use super::create_contig;
    use ::data::types::{Graph, VecArc, K_SIZE};
    use ::data::read_slice::ReadSlice;
    use ::data::edges::Edges;
    use std::sync::Arc;
    use std::cell::RefCell;
    use ::rand;
    use ::rand::Rng;

    // #[test]
    // fn simple_straight_contig() {
    // let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
    // let mut graph: Graph = Graph::default();
    // let output = rand::thread_rng().gen_ascii_chars().take(K_SIZE+2).collect::<String>();
    // sequences.borrow_mut().extend(output.clone().into_bytes().into_iter());
    // let rs0 = RS!(sequences, 0);
    // let rs1 = RS!(sequences, 1);
    // let rs2 = RS!(sequences, 2);
    // graph.insert(RS!(sequences, 0), Edges::new(1));
    // graph.insert(RS!(sequences, 1), Edges::new(2));
    // graph.insert(RS!(sequences, 2), Edges::empty());
    // {
    // let e: &mut Edges = graph.get_mut(&rs1).unwrap();
    // e.in_num += 1;
    // }
    // {
    // let e: &mut Edges = graph.get_mut(&rs2).unwrap();
    // e.in_num += 1;
    // }
    // assert_eq!(graph.len(), 3);
    // let contig = create_contig(&rs0, &mut graph, sequences.clone(), false);
    // assert_eq!(graph.len(), 0);
    // assert_eq!(output, contig);
    // }

    // #[test]
    // fn simple_cycle() {
    // let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
    // let mut graph: Graph = Graph::default();
    // let output = rand::thread_rng().gen_ascii_chars().take(K_SIZE+4).collect::<String>();
    // sequences.borrow_mut().extend(output.clone().into_bytes().into_iter());
    // let rs0 = RS!(sequences, 0);
    // let rs1 = RS!(sequences, 1);
    // let rs2 = RS!(sequences, 2);
    // let rs3 = RS!(sequences, 3);
    // let rs4 = RS!(sequences, 4);
    // graph.insert(RS!(sequences, 0), Edges::new(1));
    // graph.insert(RS!(sequences, 1), Edges::new(2));
    // graph.insert(RS!(sequences, 2), Edges::new(3));
    // graph.insert(RS!(sequences, 3), Edges::new(4));
    // graph.insert(RS!(sequences, 4), Edges::new(3));
    // {
    // let e: &mut Edges = graph.get_mut(&rs1).unwrap();
    // e.in_num += 1;
    // }
    // {
    // let e: &mut Edges = graph.get_mut(&rs2).unwrap();
    // e.in_num += 1;
    // }
    // {
    // let e: &mut Edges = graph.get_mut(&rs3).unwrap();
    // e.in_num += 2;
    // }
    // {
    // let e: &mut Edges = graph.get_mut(&rs4).unwrap();
    // e.in_num += 1;
    // }
    // assert_eq!(graph.len(), 5);
    // let mut contig = create_contig(&rs0, &mut graph, sequences.clone(), false);
    // assert_eq!(graph.len(), 2);
    // assert_eq!(output[0..K_SIZE + 3].to_string(), contig);
    // assert_eq!(graph.get(&rs3).unwrap().in_num, 1);
    // assert_eq!(graph.get(&rs4).unwrap().in_num, 1);
    // let mut contig = create_contig(&rs3, &mut graph, sequences.clone(), true);
    // assert_eq!(graph.len(), 0);
    // assert_eq!(output[3..K_SIZE + 4].to_string(), contig);
    // }
}
