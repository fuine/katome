//! Create string representation of contigs out of `Graph`.
use ::data::collections::graphs::pt_graph::{EdgeIndex, NodeIndex, PtGraph};
use ::data::collections::graphs::Graph;
use ::petgraph::EdgeDirection;
use ::petgraph::algo::scc;
use ::algorithms::pruner::Clean;
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

/// Collapse `Graph` into `SerializedContigs`.
pub trait Collapsable {
    /// Collapses `Graph` into `SerializedContigs`.
    fn collapse(mut self) -> SerializedContigs;
}

/// Representation of serialized contig.
pub type SerializedContig = String;
/// Collection of serialized contigs.
pub type SerializedContigs = Vec<String>;
type Bridges = HashSet<EdgeIndex>;

impl Collapsable for PtGraph {
    fn collapse(mut self) -> SerializedContigs {
        let mut contigs: SerializedContigs = vec![];
        let mut bridges = find_bridges(&self);
        loop {
            let starting_vertices: Vec<NodeIndex> = self.externals(EdgeDirection::Incoming)
                .collect();
            if starting_vertices.is_empty() {
                break;
            }
            for v in starting_vertices {
                contigs.extend(contigs_from_vertex(&mut self, v, &mut bridges));
            }
            // this invalidates NodeIndices so we need to call it after the loop is done
            self.remove_single_vertices();
        }
        contigs.retain(|x| !x.is_empty());
        contigs
    }
}

fn find_bridges(graph: &PtGraph) -> Bridges {
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

fn contigs_from_vertex(graph: &mut PtGraph, v: NodeIndex, bridges: &mut Bridges) -> SerializedContigs {
    let mut contigs: SerializedContigs = vec![];
    let mut contig: SerializedContig = String::new();
    let mut current_vertex = v;
    let mut current_edge_index;
    loop {
        let number_of_edges = graph.out_degree(&current_vertex);
        if number_of_edges == 0 {
            contigs.push(contig.clone());
            return contigs;
        }
        current_edge_index = unwrap!(graph.first_edge(current_vertex, EdgeDirection::Outgoing));
        if number_of_edges > 1 {
            let second_edge_index =
                unwrap!(graph.next_edge(current_edge_index, EdgeDirection::Outgoing));
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
            let (source, target) = unwrap!(graph.edge_endpoints(current_edge_index));
            contig = unwrap!(graph.node_weight(source)).name();
            contig.push(unwrap!(graph.node_weight(target)).last_char());
        }
        else {
            let (_, target) = unwrap!(graph.edge_endpoints(current_edge_index));
            contig.push(unwrap!(graph.node_weight(target)).last_char());
        }
        let (_, target) = unwrap!(graph.edge_endpoints(current_edge_index));
        decrease_weight(graph, current_edge_index, bridges);
        current_vertex = target;
    }
}

fn decrease_weight(graph: &mut PtGraph, edge: EdgeIndex, bridges: &mut Bridges) {
    {
        let edge_mut = unwrap!(graph.edge_weight_mut(edge),
                               "Trying to decrease weight of non-existent edge");
        *edge_mut -= 1;
        if *edge_mut > 0 {
            return;
        }
    }
    // weight is equal to zero - edge should be removed
    let last_edge = EdgeIndex::new(graph.edge_count() - 1);
    let last_contains = bridges.contains(&last_edge);
    let current_contains = bridges.contains(&edge);
    // keep track of the possibly switched EdgeId
    // as last edge index will become current index
    if last_contains && current_contains {
        bridges.remove(&last_edge);
    }
    else if last_contains {
        bridges.insert(edge);
        bridges.remove(&last_edge);
    }
    else if current_contains {
        bridges.remove(&edge);
    }
    graph.remove_edge(edge);
}

#[cfg(test)]
mod tests {
    extern crate rand;
    pub use ::asm::SEQUENCES;
    pub use ::asm::lock::LOCK;
    pub use ::data::collections::graphs::pt_graph::PtGraph;
    pub use ::data::primitives::K1_SIZE;
    pub use ::data::read_slice::ReadSlice;
    pub use std::iter::repeat;
    pub use super::*;

    describe! collapse {
        before_each {
            // global lock on sequences for test
            let _l = LOCK.lock().unwrap();
            // initialize with random data
            let mut name = repeat('A')
                .take(K1_SIZE)
                .collect::<String>();
            name.push_str("TGC");
            {
                let mut seq = SEQUENCES.write().unwrap();
                seq.clear();
                seq.extend(name.clone().into_bytes());
                seq.extend(name.clone().into_bytes());
            }
            let mut graph: PtGraph = PtGraph::default();
            let _w = graph.add_node(ReadSlice::new(0));
            let _x = graph.add_node(ReadSlice::new(1));
            assert_eq!(graph.node_count(), 2);
        }

        it "doesn't create any contig" {
            assert_eq!(graph.edge_count(), 0);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 0);
        }

        it "creates one small contig" {
            graph.add_edge(_w, _x, 1);
            assert_eq!(graph.edge_count(), 1);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            assert_eq!(contigs[0].as_str(), &name[..K1_SIZE+1]);
        }

        it "creates one longer contig" {
            let y = graph.add_node(ReadSlice::new(2));
            let z = graph.add_node(ReadSlice::new(3));
            graph.add_edge(_w, _x, 1);
            graph.add_edge(_x, y, 1);
            graph.add_edge(y, z, 1);
            assert_eq!(graph.edge_count(), 3);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            assert_eq!(contigs[0].as_str(), &name[..K1_SIZE+3]);
        }

        it "creates two contigs" {
            let y = graph.add_node(ReadSlice::new(2));
            let z = graph.add_node(ReadSlice::new(3));
            graph.add_edge(_w, _x, 1);
            graph.add_edge(y, z, 1);
            assert_eq!(graph.edge_count(), 2);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 2);
            assert_eq!(contigs[0].as_str(), &name[..K1_SIZE+1]);
            assert_eq!(contigs[1].as_str(), &name[2..K1_SIZE+3]);
        }

        it "creates two longer contigs" {
            let y = graph.add_node(ReadSlice::new(2));
            let z = graph.add_node(ReadSlice::new(3));
            graph.add_edge(_w, _x, 2);
            graph.add_edge(_x, y, 1);
            graph.add_edge(_x, z, 1);
            assert_eq!(graph.edge_count(), 3);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 2);
            assert_eq!(contigs[0].as_str(), &name[..K1_SIZE+2]);
            let mut n = name[..K1_SIZE+1].to_string();
            n.push(name.chars().nth(K1_SIZE+2).unwrap());
            assert_eq!(contigs[1], n);
        }

        it "deals with simple cycle" {
            let y = graph.add_node(ReadSlice::new(2));
            let z = graph.add_node(ReadSlice::new(3));
            graph.add_edge(_w, _x, 1);
            graph.add_edge(_x, y, 1);
            graph.add_edge(y, z, 1);
            graph.add_edge(z, _x, 1);
            assert_eq!(graph.edge_count(), 4);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            let mut n = name[..K1_SIZE+3].to_string();
            n.push(name.chars().nth(K1_SIZE).unwrap());
            assert_eq!(contigs[0], n);
        }
    }
}
