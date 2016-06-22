use ::data::types::{Graph, VecArc, VertexId};
use ::data::read_slice::ReadSlice;
use ::algorithms::hardener::{remove_not_connected_vertices};
use ::asm::assembler::{print_stats};
use std::fs::File;
use std::error::Error;
use std::io::Write;

pub fn collapse(graph: &mut Graph, vec: VecArc, filename: String) {
    // open file to writing
    let mut fh = match File::create(&filename) {
        Err(why) => panic!("Couldn't create output file {}: {}",
                           filename,
                           why.description()),
        Ok(file) => file,
    };
    // find input indices
    while !graph.is_empty() {
        let starting_vertices = get_starting_vertices(graph);
        // for each input index
        for starting_vertex in starting_vertices {
            let mut seq_to_write: String;
            // if first -> write whole sequence
            let mut next_edge: VertexId;
            let mut vertex: ReadSlice;
            {
                let mut starting_edges = graph.get_mut(&starting_vertex)
                    .expect("Couldn't find vertex which is supposed to be in the graph.");
                seq_to_write = starting_vertex.name();
                next_edge = starting_edges.outgoing[0].0;
                vertex = RS!(vec, next_edge);
                // delete this edge
                starting_edges.outgoing[0].1 -= 1;
                if starting_edges.outgoing[0].1 == 0 { // end vertex
                    // remove empty outgoing edges
                    starting_edges.remove_edge(0);
                }
            }
            loop {
                let mut edges = graph.get_mut(&vertex)
                    .expect("Couldn't find vertex which is supposed to be in the graph.");
                seq_to_write.push(vertex.last_char());
                if edges.in_num == 0 {
                    panic!("WOT TEH SHIT");
                }
                edges.in_num -= 1;
                if edges.outgoing.is_empty() || edges.outgoing.len() > 1 { break; }
                // remove edge
                edges.outgoing[0].1 -= 1;
                if edges.outgoing[0].1 == 0 { // end vertex
                    // remove empty outgoing
                    edges.remove_edge(0);
                }
                next_edge = edges.outgoing[0].0;
                vertex = RS!(vec, next_edge);
            }
            // remove dead indices
            remove_not_connected_vertices(graph);
            seq_to_write.push('\n');
            match fh.write_all(seq_to_write.as_bytes()) {
                Ok(_) => (),
                Err(e) => error!("Couldn't write sequence {} to file: {}", seq_to_write, e),
            }
        }
        print_stats(graph);
    }
}


fn get_starting_vertices(graph: &mut Graph) -> Vec<ReadSlice>{
    graph.iter()
         .filter(|&(_, val)| val.in_num == 0)
         .map(|(key, _)| key.clone())
         .collect::<Vec<ReadSlice>>()
}
