use ::data::types::{Graph, VertexId};
use ::petgraph::graph::{NodeIndex, EdgeIndex};
use ::petgraph::EdgeDirection;
use ::petgraph::algo::scc;
use ::algorithms::pruner::remove_single_vertices;
use std::iter;
use std::collections::HashSet;
/* contigs <- 0 // {output of assembler}
v <- v0 // current vertex
c <- 0  // current contig
loop
    if |v.out| == 0 then
        contigs.insert(c)
        return contigs // end of algorithm
    end if
    e <- v.out[0] // current edge
    if|v:out| > 1 then
        f <- v.out[1] // second current edge
        if|v.out| == 2 and (isBridge(e) or isBridge(f)) then
            if isBridge(e)then
                e <- f
            end if
        else
            contigs.insert(c) // v is ambiguous
            c <- 0 // new contig
        end if
    end if
    c.insert(e)
    v <- e.target
    G.delete(e) // decreases edge's weight, if it achieves 0, remove e from G
end loop */

pub type Contig = String;
pub type Contigs = Vec<String>;
type Bridges = HashSet<EdgeIndex<VertexId>>;

pub fn euler_paths(graph: &mut Graph) -> Contigs {
    let mut contigs: Contigs = vec![];
    let bridges = find_bridges(graph);
    loop {
        let starting_vertices: Vec<NodeIndex<VertexId>> = graph.externals(EdgeDirection::Incoming).collect();
        if starting_vertices.is_empty() {
            break;
        }
        for v in starting_vertices {
            contigs.extend(euler_paths_for_vertex(graph, v, &bridges));
        }
    }
    contigs
}

fn find_bridges(graph: &Graph) -> Bridges {
    info!("Start finding bridges");
    let sccs = scc(graph);
    let mut vec = iter::repeat(0).take(graph.node_count()).collect::<Vec<usize>>();
    let count: usize = sccs.iter().map(|x| x.iter().count()).sum();
    debug!("Nodes: {} sccs: {} vec: {}", graph.node_count(), count, vec.len());
    // flatten sccs representation from petgraph
    for (i, scc) in sccs.into_iter().enumerate() {
        for node in scc.into_iter() {
            vec[node.index()] = i;
        }
    }
    let mut bridges = Bridges::new();
    for (id, edge) in graph.raw_edges().iter().enumerate() {
        if vec[edge.source().index()] != vec[edge.target().index()] {
            bridges.insert(EdgeIndex::new(id));
        }
    }
    info!("{} bridges found", bridges.len());
    bridges
}

fn euler_paths_for_vertex(graph: &mut Graph, v: NodeIndex<VertexId>, bridges: &Bridges) -> Contigs {
    let mut contigs: Contigs = vec![];
    let mut contig: Contig = String::new();
    let mut current_vertex = v;
    let mut current_edge_index;
    loop {
        let number_of_edges = graph.neighbors_directed(current_vertex, EdgeDirection::Outgoing).take(3).count();
        if number_of_edges == 0 {
            contigs.push(contig.clone());
            remove_single_vertices(graph);
            return contigs;
        }
        current_edge_index = graph.first_edge(current_vertex, EdgeDirection::Outgoing).expect("No edge, despite count being higher than 0");
        if number_of_edges > 1 {
            let second_edge_index = graph.next_edge(current_edge_index, EdgeDirection::Outgoing).expect("No second edge, despite count being higher than 1");
            let first_bridge = bridges.contains(&current_edge_index);
            if number_of_edges == 2 && (first_bridge || bridges.contains(&second_edge_index)) {
                if first_bridge {
                    current_edge_index = second_edge_index;
                }
            }
            else {
                contigs.push(contig.clone());
                contig.clear();
            }
        }
        if contig.is_empty() {
            let (source, target) = graph.edge_endpoints(current_edge_index).expect("This should never fail");
            contig = graph.node_weight(source).unwrap().name();
            contig.push(graph.node_weight(target).unwrap().last_char());
        }
        else {
            let (_, target) = graph.edge_endpoints(current_edge_index).expect("This should never fail");
            contig.push(graph.node_weight(target).unwrap().last_char());
        }
        let (_, target) = graph.edge_endpoints(current_edge_index).expect("Trying to read non-existent edge");
        decrease_weight(graph, current_edge_index);
        current_vertex = target;
    }
}

fn decrease_weight(graph: &mut Graph, edge: EdgeIndex<VertexId>) {
    {
        let edge_mut = graph.edge_weight_mut(edge).expect("Trying to decrease weight of non-existent edge");
        *edge_mut -= 1;
        if *edge_mut > 0 {
            return;
        }
    }
    // weight is equal to zero - edge should be removed
    // let (source, target) = graph.edge_endpoints(edge).expect("Trying to delete non-existent edge");
    graph.remove_edge(edge);
    // if let None = graph.neighbors_undirected(source).next() {
        // graph.remove_node(source);
    // }
    // if let None = graph.neighbors_undirected(target).next() {
        // graph.remove_node(target);
    // }
}
