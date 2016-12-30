//! Create string representation of contigs out of `Graph`.

use algorithms::shrinker::Shrinkable;
use collections::Graph;
use collections::graphs::pt_graph::{EdgeIndex, NodeIndex, PtGraph};
use slices::BasicSlice;

use fixedbitset::FixedBitSet;

use petgraph::EdgeDirection;
use petgraph::algo::{connected_components, tarjan_scc};
use petgraph::visit::EdgeRef;

/// Collapse `Graph` into `SerializedContigs`.
pub trait Collapsable: Shrinkable {
    /// Collapses `Graph` into `SerializedContigs`.
    fn collapse(self) -> SerializedContigs;
}

/// Representation of serialized contig.
pub type SerializedContig = String;
/// Collection of serialized contigs.
pub type SerializedContigs = Vec<String>;

impl Collapsable for PtGraph {
    fn collapse(mut self) -> SerializedContigs {
        let mut contigs: SerializedContigs = vec![];
        info!("Starting collapse of the graph");
        // ensure that we don't end up with straight paths longer than
        // one edge
        self.shrink();
        let node_count = self.node_count();
        let mut ambiguous_nodes = FixedBitSet::with_capacity(node_count);
        let mut single_vertices: Vec<NodeIndex> = vec![];
        info!("Graph has {} weakly connected components", connected_components(&self));
        loop {
            // this is a loop over nodes which have in_degree == 0
            loop {
                // get all starting nodes
                let externals = self.externals(EdgeDirection::Incoming).collect::<Vec<NodeIndex>>();
                if externals.is_empty() {
                    break;
                }
                // create contigs from each starting node
                for n in externals {
                    let contigs_ = contigs_from_vertex(&mut self,
                                                       n,
                                                       &mut ambiguous_nodes,
                                                       &mut single_vertices);
                    contigs.extend(contigs_);
                }
                // remove fake starting nodes (nodes with in_degree == out_degree == 0
                // self.remove_single_vertices();
                remove_single_with_ambiguity(&mut self, &mut single_vertices, &mut ambiguous_nodes);
            }

            // Cycle in the input -- use dfspostorder to get the starting node
            // in the cycle. At this point no node in the graph has in_degree == 0
            // and so we need to find a starting node somewhere in the 'highest'
            // cycle in terms of topology of the subgraph.
            if self.node_count() != 0 {
                // we guarantee that there's at least one node to unwrap here
                let node_in_cycle = unwrap!(tarjan_scc(&self).iter().last())[0];
                contigs.extend(contigs_from_vertex(&mut self,
                                                   node_in_cycle,
                                                   &mut ambiguous_nodes,
                                                   &mut single_vertices));
                remove_single_with_ambiguity(&mut self, &mut single_vertices, &mut ambiguous_nodes);
            }
            else {
                break;
            }

        }
        trace!("{} nodes left in the graph after collapse", self.node_count());
        info!("Collapse ended. Created {} contigs which have {} nucleotides",
              contigs.len(),
              contigs.iter().map(|x| x.len()).sum::<usize>());
        contigs
    }
}

/// Remove given nodes, but save information about ambiguity in the process.
#[inline]
fn remove_single_with_ambiguity(graph: &mut PtGraph, to_remove: &mut Vec<NodeIndex>,
                                ambiguous_nodes: &mut FixedBitSet) {
    // reverse sort node indices such that removal won't cause any swaps withing to_remove
    to_remove.sort_by(|a, b| b.cmp(a));
    let mut last_node = graph.node_count();
    for node in to_remove.drain(..) {
        last_node -= 1;
        // copy information about ambiguity of the last node
        ambiguous_nodes.copy_bit(last_node, node.index());
        graph.remove_node(node);
    }
}

#[inline]
fn contigs_from_vertex(graph: &mut PtGraph, v: NodeIndex, ambiguous_nodes: &mut FixedBitSet,
                       single_vertices: &mut Vec<NodeIndex>)
                       -> SerializedContigs {
    let mut contigs: SerializedContigs = vec![];
    let mut contig: SerializedContig = String::new();
    let mut current_vertex = v;
    let mut current_edge_index;
    let mut simple_loop_;
    let mut num_in = graph.in_degree(current_vertex);
    let mut num_out = graph.out_degree(current_vertex);
    loop {
        simple_loop_ = None;
        if num_out == 0 {
            if num_in == 0 {
                single_vertices.push(current_vertex);
            }
            if !contig.is_empty() {
                contigs.push(contig.clone());
            }
            return contigs;
        }
        current_edge_index = unwrap!(graph.first_edge(current_vertex, EdgeDirection::Outgoing),
                                     "{:?} out: {}",
                                     current_vertex,
                                     num_out);
        if ambiguous_nodes.contains(current_vertex.index()) {
            if !contig.is_empty() {
                contigs.push(contig.clone());
                contig.clear();
            }
        }
        else {
            match (num_in, num_out) {
                (2, 1) => {
                    // make sure that we are not dealing with the loopy end
                    // if it is a self-loop then it must be of shape
                    // a -> b, b -> b
                    if self_loop(graph, current_vertex).is_none() {
                        simple_loop_ = simple_loop(graph, current_edge_index);
                        if simple_loop_.is_none() {
                            ambiguous_nodes.insert(current_vertex.index());
                            if !contig.is_empty() {
                                contigs.push(contig.clone());
                                contig.clear();
                            }
                        }
                    }
                }
                (1, 2) | (2, 2) => {
                    // because we handle simple loops in the match arm for 1 then
                    // any vertex with 2 outgoing edges has either a self-loop or
                    // is ambiguous
                    if let Some(e) = self_loop(graph, current_vertex) {
                        current_edge_index = e;
                    }
                    else {
                        ambiguous_nodes.insert(current_vertex.index());
                        if !contig.is_empty() {
                            contigs.push(contig.clone());
                            contig.clear();
                        }
                    }
                }
                (0, 1) | (1, 1) => {}
                _ => {
                    // ambiguous edge
                    ambiguous_nodes.insert(current_vertex.index());
                    if !contig.is_empty() {
                        contigs.push(contig.clone());
                        contig.clear();
                    }
                }
            }
        }
        if contig.is_empty() {
            contig = unwrap!(graph.edge_weight(current_edge_index)).0.name();
        }
        else {
            contig.push_str(&unwrap!(graph.edge_weight(current_edge_index)).0.remainder());
        }
        let (_, target) = unwrap!(graph.edge_endpoints(current_edge_index));
        num_in = graph.in_degree(target);
        if let Some(e) = simple_loop_ {
            contig.push_str(&unwrap!(graph.edge_weight(e)).0.remainder());
            // make sure to possibly remove edges in the right order (petgraph
            // will switch the index of the last edge is anything prior to it is
            // removed)
            if current_edge_index < e {
                decrease_weight(graph, e);
                decrease_weight(graph, current_edge_index);
            }
            else {
                decrease_weight(graph, current_edge_index);
                decrease_weight(graph, e);
            }
        }
        else {
            decrease_weight(graph, current_edge_index);
        }
        num_out = graph.out_degree(target);
        // if old current_vertex became a single node it's time to remove it
        if graph.in_degree(current_vertex) == 0 && graph.out_degree(current_vertex) == 0 {
            single_vertices.push(current_vertex);
        }
        current_vertex = target;
    }
}

/// Check if node has self-loop.
///
/// Only loops that are allowed look like this:
/// a -> b -> c, b -> b
/// a -> b, b -> b
/// It is assumed that node has 1 or 2 outgoing edges
/// TODO add simple diagram to illustrate
#[inline]
fn self_loop(graph: &PtGraph, node: NodeIndex) -> Option<EdgeIndex> {
    if graph.in_degree(node) > 2 {
        return None;
    }
    for e in graph.edges_directed(node, EdgeDirection::Outgoing) {
        if e.target() == e.source() {
            return Some(e.id());
        }
    }
    None
}

/// Check if node has simple loop.
///
/// Simple loop has only one entrance and one exit.
/// Function does not deal well with self loops.
/// Note that shrinking enforces only two nodes in the loop.
/// Edges source has to have exactly one outgoing edge (functions argument).
/// TODO add dragram
#[inline]
fn simple_loop(graph: &PtGraph, edge: EdgeIndex) -> Option<EdgeIndex> {
    let (source, target) = unwrap!(graph.edge_endpoints(edge));
    let in_source = graph.in_degree(source);
    if in_source == 0 || in_source > 2 {
        return None;
    }
    let in_target = graph.in_degree(target);
    let out_target = graph.out_degree(target);
    // since this isn't self loop these conditions must hold
    // note that shrinking ensures that we won't have situation like this:
    // a -> b -> c, b -> d, d -> b
    if in_target != 1 || out_target != 2 {
        return None;
    }
    for e in graph.edges_directed(target, EdgeDirection::Outgoing) {
        // if weight of the edge going to the target doesn't have higher weight
        // then such loop would be broken at the source and contig would never
        // reach to the specified target. This in turns leads to
        // misassemblies/mismatches in the final contigs.
        if e.target() == source && e.weight().1 < unwrap!(graph.edge_weight(edge)).1 {
            return Some(e.id());
        }
    }
    None
}

#[inline]
fn decrease_weight(graph: &mut PtGraph, edge: EdgeIndex) {
    {
        let edge_mut = unwrap!(graph.edge_weight_mut(edge),
                               "Trying to decrease weight of non-existent edge");
        edge_mut.1 -= 1;
        if edge_mut.1 > 0 {
            return;
        }
    }
    // weight is equal to zero - edge should be removed
    graph.remove_edge(edge);
}

#[cfg(test)]
mod tests {
    use ::asm::SEQUENCES;
    use ::asm::lock::LOCK;
    use ::collections::graphs::pt_graph::PtGraph;
    use ::compress::compress_edge;
    use ::prelude::{K1_SIZE, K_SIZE};
    use ::slices::{BasicSlice, EdgeSlice};
    use std::iter::repeat;
    use super::*;
    use std::panic::catch_unwind;

    macro_rules! setup {
        ($l:ident, $g:ident, $n:ident, $s:ident, $w:ident, $x:ident) => {
            // global lock on sequences for test
            let $l = LOCK.lock().unwrap();
            let mut $n = repeat('A')
                .take(unsafe{K1_SIZE})
                .collect::<String>();
            let mut $s = $n.clone();
            $n.push_str("TGCT");
            $s.push_str("G");
            let c1 = compress_edge($n[..unsafe{K_SIZE}].as_bytes());
            let c2 = compress_edge($n[1..unsafe{K_SIZE}+1].as_bytes());
            let c3 = compress_edge($n[2..unsafe{K_SIZE}+2].as_bytes());
            let c4 = compress_edge($n[3..unsafe{K_SIZE}+3].as_bytes());
            let c5 = compress_edge($s[..unsafe{K_SIZE}].as_bytes());
            {
                let mut seq = SEQUENCES.write();
                seq.clear();
                seq.push(vec![].into_boxed_slice());
                seq.push(c1.into_boxed_slice());
                seq.push(c2.into_boxed_slice());
                seq.push(c3.into_boxed_slice());
                seq.push(c4.into_boxed_slice());
                seq.push(c5.into_boxed_slice());
            }
            let mut $g: PtGraph = PtGraph::default();
            let $w = $g.add_node(());
            let $x = $g.add_node(());
            assert_eq!($g.node_count(), 2);
        }
    }

    macro_rules! test {
        ($n:ident, $b:block) => {
            #[test]
            fn $n() {
                let result = {
                    $b
                };
                assert!(result.is_ok());
            }
        }
    }

    test!(doesnt_create_any_contig, {
        setup!(_l, graph, name, _second, _w, _x);
        catch_unwind(|| {
            assert_eq!(graph.edge_count(), 0);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 0);
        })
    });

    test!(creates_one_small_contig, {
        setup!(_l, graph, name, _second, _w, _x);
        catch_unwind(|| {
            graph.add_edge(_w, _x, (EdgeSlice::new(1), 1));
            assert_eq!(graph.edge_count(), 1);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            assert_eq!(contigs[0].as_str(), &name[..unsafe { K_SIZE }]);
        })
    });

    test!(creates_one_longer_contig, {
        setup!(_l, graph, name, _second, _w, _x);
        catch_unwind(|| {
            let y = graph.add_node(());
            let z = graph.add_node(());
            graph.add_edge(_w, _x, (EdgeSlice::new(1), 1));
            graph.add_edge(_x, y, (EdgeSlice::new(2), 1));
            graph.add_edge(y, z, (EdgeSlice::new(3), 1));
            assert_eq!(graph.edge_count(), 3);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            assert_eq!(contigs[0].as_str(), &name[..unsafe { K_SIZE } + 2]);
        })
    });

    test!(creates_two_contigs, {
        setup!(_l, graph, name, _second, _w, _x);
        catch_unwind(|| {
            let y = graph.add_node(());
            let z = graph.add_node(());
            graph.add_edge(_w, _x, (EdgeSlice::new(1), 1));
            graph.add_edge(y, z, (EdgeSlice::new(3), 1));
            assert_eq!(graph.edge_count(), 2);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 2);
            assert_eq!(contigs[0].as_str(), &name[..unsafe { K_SIZE }]);
            assert_eq!(contigs[1].as_str(), &name[2..unsafe { K_SIZE } + 2]);
        })
    });

    test!(creates_two_longer_contigs, {
        setup!(_l, graph, name, second, _w, _x);
        catch_unwind(|| {
            let y = graph.add_node(());
            let z = graph.add_node(());
            graph.add_edge(_w, _x, (EdgeSlice::new(1), 2));
            graph.add_edge(_x, y, (EdgeSlice::new(2), 1));
            graph.add_edge(_x, z, (EdgeSlice::new(5), 1));
            assert_eq!(graph.edge_count(), 3);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 4);
            assert_eq!(contigs[0].as_str(), &name[..unsafe { K_SIZE }]);
            assert_eq!(contigs[2].as_str(), &name[..unsafe { K_SIZE }]);
            assert_eq!(contigs[1], second);
            assert_eq!(contigs[3], &name[1..unsafe { K_SIZE } + 1]);
        })
    });

    // w -> x -> y -> z -> x
    test!(deals_with_simple_cycle, {
        setup!(_l, graph, name, _second, _w, _x);
        catch_unwind(|| {
            let y = graph.add_node(());
            let z = graph.add_node(());
            graph.add_edge(_w, _x, (EdgeSlice::new(1), 1));
            graph.add_edge(_x, y, (EdgeSlice::new(2), 1));
            graph.add_edge(y, z, (EdgeSlice::new(3), 1));
            graph.add_edge(z, _x, (EdgeSlice::new(4), 1));
            assert_eq!(graph.edge_count(), 4);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            assert_eq!(contigs[0], name);
        })
    });

    // w -> x -> y -> x
    test!(collapses_self_loop, {
        setup!(_l, graph, name, _second, _w, _x);
        catch_unwind(|| {
            let y = graph.add_node(());
            graph.add_edge(_w, _x, (EdgeSlice::new(1), 1));
            graph.add_edge(_x, y, (EdgeSlice::new(2), 2));
            graph.add_edge(y, _x, (EdgeSlice::new(3), 2));
            // graph.add_edge(z, _x, (EdgeSlice::new(4), 1));
            assert_eq!(graph.edge_count(), 3);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            assert_eq!(contigs[0], format!("{}GCGC", &name[..unsafe { K_SIZE }]));
        })
    });

    // w -> x -> z, x -> y -> x
    test!(collapses_simple_loop, {
        setup!(_l, graph, name, _second, _w, _x);
        catch_unwind(|| {
            let y = graph.add_node(());
            let z = graph.add_node(());
            graph.add_edge(_w, _x, (EdgeSlice::new(1), 1));
            graph.add_edge(_x, y, (EdgeSlice::new(2), 2));
            graph.add_edge(y, _x, (EdgeSlice::new(3), 2));
            graph.add_edge(_x, z, (EdgeSlice::new(4), 1));
            assert_eq!(graph.edge_count(), 4);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            assert_eq!(contigs[0], format!("{}GCGCT", &name[..unsafe { K_SIZE }]));
        })
    });

    // w -> x -> z, x -> y -> x, w -> k -> w
    test!(collapses_non_tree_graph, {
        setup!(_l, graph, name, _second, _w, _x);
        catch_unwind(|| {
            let y = graph.add_node(());
            let z = graph.add_node(());
            let k = graph.add_node(());
            graph.add_edge(_w, _x, (EdgeSlice::new(1), 1));
            graph.add_edge(_x, y, (EdgeSlice::new(2), 2));
            graph.add_edge(y, _x, (EdgeSlice::new(3), 2));
            graph.add_edge(_x, z, (EdgeSlice::new(4), 1));
            graph.add_edge(_w, k, (EdgeSlice::new(5), 1));
            graph.add_edge(k, _w, (EdgeSlice::new(5), 1));
            assert_eq!(graph.edge_count(), 6);
            let contigs = graph.collapse();
            assert_eq!(contigs.len(), 1);
            assert_eq!(contigs[0], format!("{}GGTGCGCT", &name[..unsafe { K1_SIZE }]));
        })
    });
}
