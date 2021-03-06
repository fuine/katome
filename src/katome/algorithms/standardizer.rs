//! Graph standardization module.
//!
//! Standardization of the `Graph` is a process in which weights of edges are
//! standardized accordingly.
//!
//! In the first phase of the standardization procedure contigs are
//! standardized. For each contig mean weight is calculated and all edges which
//! belong to such contig have their weights set to the mean value. This reduces
//! the number of short contigs created in the output.
//!
//! Secondly all edges should be standardized with respect to
//! the calculated ratio, described by this formula:
//!
//! `(original_genome_length - k_size)  / (sum_of_all_weights - sum_of_weights_lower_than_threshold)`
//!
//! During this process all edges with weights lower than given threshold will
//! be removed from the graph.


use algorithms::pruner::Clean;
use collections::Graph;
use collections::graphs::pt_graph::{EdgeIndex, NodeIndex, PtGraph};
use prelude::EdgeWeight;

use petgraph::EdgeDirection;

/// Contig representation.
type Contig = Vec<EdgeIndex>;
/// Vector of `Contig`s.
type GraphContigs = Vec<Contig>;

/// Trait describing standardization of the `Graph`.
pub trait Standardizable {
    /// Standardize edges of the `Graph`.
    fn standardize_edges(&mut self, original_genome_length: usize, k_size: usize,
                         threshold: EdgeWeight);
    /// Standardize each contig.
    fn standardize_contigs(&mut self);
}

impl Standardizable for PtGraph {
    fn standardize_edges(&mut self, original_genome_length: usize, k_size: usize,
                         threshold: EdgeWeight) {
        // calculate sum of all weights of edges (s) and sum of weights lower than threshold (l)
        let (s, l) = self.raw_edges()
            .iter()
            .fold((0_usize, 0_usize), |acc, e| {
                if e.weight.1 < threshold {
                    (acc.0 + e.weight.1 as usize, acc.1 + e.weight.1 as usize)
                }
                else {
                    (acc.0 + e.weight.1 as usize, acc.1)
                }
            });
        let p: f64 =
            calculate_standardization_ratio(original_genome_length, k_size, s as usize, l as usize);
        info!("Ratio: {} for g: {} k: {} s: {} l: {}", p, original_genome_length, k_size, s, l);
        // normalize edges across the graph
        for weight in self.edge_weights_mut() {
            let new_weight = (weight.1 as f64 * p).round() as EdgeWeight;
            weight.1 = if new_weight == 0 && weight.1 >= threshold {
                1
            }
            else {
                new_weight
            };
        }
        // remove edges with weight 0
        self.remove_weak_edges(1);
    }

    fn standardize_contigs(&mut self) {
        let ambiguous_nodes = self.get_ambiguous_nodes();
        info!("Found {} ambiguous nodes", ambiguous_nodes.len());
        for node in &ambiguous_nodes {
            let contigs = get_contigs_from_node(self, *node, &ambiguous_nodes);
            for contig in contigs {
                standardize_contig(self, contig);
            }
        }
    }
}

#[inline]
fn get_contigs_from_node(graph: &PtGraph, starting_node: NodeIndex,
                         ambiguous_nodes: &<PtGraph as Graph>::AmbiguousNodes)
                         -> GraphContigs {
    let mut contigs = vec![];
    for node in graph.neighbors_directed(starting_node, EdgeDirection::Outgoing) {
        let mut contig = vec![];
        let mut current_node = node;
        let mut current_edge = unwrap!(graph.find_edge(starting_node, node));
        loop {
            let out_degree = graph.out_degree(current_node);
            contig.push(current_edge);
            if out_degree != 1 || ambiguous_nodes.contains(&current_node) {
                break;
            }
            current_edge = unwrap!(graph.first_edge(current_node, EdgeDirection::Outgoing));
            current_node = unwrap!(graph.edge_endpoints(current_edge)).1;
        }
        contigs.push(contig);
    }
    contigs
}

// Set weights of consecutive `Edge`s in the `Contig` to the mean value
#[inline]
fn standardize_contig(graph: &mut PtGraph, contig: Contig) {
    // sum all weights in the contig
    let sum: usize = contig.iter()
        .map(|&e| unwrap!(graph.edge_weight(e)).1 as usize)
        .sum();
    // calculate new, standardizes weight
    let standardized_weight = (sum as f64 / contig.len() as f64).round() as EdgeWeight;
    // modify all edges in the contig with new value
    for edge in contig {
        unwrap!(graph.edge_weight_mut(edge)).1 = standardized_weight;
    }
}

#[inline]
fn calculate_standardization_ratio(original_genome_length: usize, k: usize,
                                   sum_of_all_weights: usize, weights_lower_than_threshold: usize)
                                   -> f64 {
    (original_genome_length - k) as f64 / (sum_of_all_weights - weights_lower_than_threshold) as f64
}


#[cfg(test)]
mod tests {
    pub use ::collections::graphs::pt_graph::{EdgeIndex, PtGraph};
    pub use ::slices::EdgeSlice;
    pub use super::*;
    use super::calculate_standardization_ratio;

    #[test]
    fn calculates_standardization_ratio() {
        let p = calculate_standardization_ratio(10, 0, 10, 0);
        assert_eq!(p, 1.0);
    }

    mod standardize_contig {
        use super::*;
        #[test]
        fn standardizes_contigs_in_empty_graph() {
            let mut g = PtGraph::default();
            assert_eq!(g.node_count(), 0);
            assert_eq!(g.edge_count(), 0);
            g.standardize_contigs();
            assert_eq!(g.node_count(), 0);
            assert_eq!(g.edge_count(), 0);
        }

        #[test]
        fn standardizes_single_contig() {
            let mut graph = PtGraph::default();
            let x = graph.add_node(());
            let y = graph.add_node(());
            let z = graph.add_node(());
            let e1 = graph.add_edge(x, y, (EdgeSlice::default(), 100));
            let e2 = graph.add_edge(y, z, (EdgeSlice::default(), 1));

            assert_eq!(graph.edge_weight(e1).unwrap().1, 100);
            assert_eq!(graph.edge_weight(e2).unwrap().1, 1);
            graph.standardize_contigs();
            assert_eq!(graph.edge_count(), 2);
            assert_eq!(graph.edge_weight(e1).unwrap().1, 51);
            assert_eq!(graph.edge_weight(e2).unwrap().1, 51);
        }

        #[test]
        fn standardizes_contig_one_in_two_out() {
            let mut graph = PtGraph::from_edges(&[(0, 1, (EdgeSlice::default(), 8)),
                                                  (1, 2, (EdgeSlice::default(), 4)),
                                                  (2, 3, (EdgeSlice::default(), 115)),
                                                  (3, 4, (EdgeSlice::default(), 1)),
                                                  (2, 5, (EdgeSlice::default(), 2)),
                                                  (5, 6, (EdgeSlice::default(), 4)),
                                                  (6, 7, (EdgeSlice::default(), 9))]);
            graph.standardize_contigs();
            assert_eq!(graph.edge_count(), 7);
            assert_eq!(graph.edge_weight(EdgeIndex::new(0)).unwrap().1, 6);
            assert_eq!(graph.edge_weight(EdgeIndex::new(1)).unwrap().1, 6);
            assert_eq!(graph.edge_weight(EdgeIndex::new(2)).unwrap().1, 58);
            assert_eq!(graph.edge_weight(EdgeIndex::new(3)).unwrap().1, 58);
            for i in 4..7 {
                assert_eq!(graph.edge_weight(EdgeIndex::new(i)).unwrap().1, 5);
            }
        }

        #[test]
        fn standardizes_contigs_two_in_one_out() {
            let mut graph = PtGraph::from_edges(&[(0, 1, (EdgeSlice::default(), 8)),
                                                  (1, 2, (EdgeSlice::default(), 4)),
                                                  (2, 3, (EdgeSlice::default(), 115)),
                                                  (3, 4, (EdgeSlice::default(), 1)),
                                                  (7, 2, (EdgeSlice::default(), 2)),
                                                  (5, 6, (EdgeSlice::default(), 4)),
                                                  (6, 7, (EdgeSlice::default(), 9))]);
            graph.standardize_contigs();
            assert_eq!(graph.edge_count(), 7);
            assert_eq!(graph.edge_weight(EdgeIndex::new(0)).unwrap().1, 6);
            assert_eq!(graph.edge_weight(EdgeIndex::new(1)).unwrap().1, 6);
            assert_eq!(graph.edge_weight(EdgeIndex::new(2)).unwrap().1, 58);
            assert_eq!(graph.edge_weight(EdgeIndex::new(3)).unwrap().1, 58);
            for i in 4..7 {
                assert_eq!(graph.edge_weight(EdgeIndex::new(i)).unwrap().1, 5);
            }
        }

        #[test]
        fn standardizes_contigs_two_in_two_out() {
            let mut graph = PtGraph::from_edges(&[(0, 1, (EdgeSlice::default(), 8)),
                                                  (1, 2, (EdgeSlice::default(), 4)),
                                                  (2, 3, (EdgeSlice::default(), 115)),
                                                  (3, 4, (EdgeSlice::default(), 1)),
                                                  (7, 2, (EdgeSlice::default(), 2)),
                                                  (5, 6, (EdgeSlice::default(), 4)),
                                                  (6, 7, (EdgeSlice::default(), 9)),
                                                  (2, 8, (EdgeSlice::default(), 178)),
                                                  (8, 9, (EdgeSlice::default(), 298)),
                                                  (9, 10, (EdgeSlice::default(), 123)),
                                                  (10, 11, (EdgeSlice::default(), 9128))]);
            graph.standardize_contigs();
            assert_eq!(graph.edge_count(), 11);
            assert_eq!(graph.edge_weight(EdgeIndex::new(0)).unwrap().1, 6);
            assert_eq!(graph.edge_weight(EdgeIndex::new(1)).unwrap().1, 6);
            assert_eq!(graph.edge_weight(EdgeIndex::new(2)).unwrap().1, 58);
            assert_eq!(graph.edge_weight(EdgeIndex::new(3)).unwrap().1, 58);
            for i in 4..7 {
                assert_eq!(graph.edge_weight(EdgeIndex::new(i)).unwrap().1, 5);
            }
            for i in 7..11 {
                assert_eq!(graph.edge_weight(EdgeIndex::new(i)).unwrap().1, 2432);
            }
        }

        #[test]
        fn standardizes_contigs_in_cycle() {
            let mut graph = PtGraph::from_edges(&[(0, 1, (EdgeSlice::default(), 8)),
                                                  (1, 2, (EdgeSlice::default(), 4)),
                                                  (2, 3, (EdgeSlice::default(), 115)),
                                                  (3, 1, (EdgeSlice::default(), 1))]);
            graph.standardize_contigs();
            assert_eq!(graph.edge_count(), 4);
            assert_eq!(graph.edge_weight(EdgeIndex::new(0)).unwrap().1, 8);
            for i in 1..4 {
                assert_eq!(graph.edge_weight(EdgeIndex::new(i)).unwrap().1, 40);
            }
        }

        #[test]
        fn standardizes_edges_from_example() {
            let mut graph = PtGraph::from_edges(&[(0, 1, (EdgeSlice::default(), 8)),
                                                  (1, 2, (EdgeSlice::default(), 8)),
                                                  (2, 3, (EdgeSlice::default(), 8)),
                                                  (3, 4, (EdgeSlice::default(), 16)),
                                                  (4, 5, (EdgeSlice::default(), 16)),
                                                  (5, 6, (EdgeSlice::default(), 9)),
                                                  (6, 7, (EdgeSlice::default(), 9)),
                                                  (7, 8, (EdgeSlice::default(), 1)),
                                                  (7, 9, (EdgeSlice::default(), 4)),
                                                  (9, 10, (EdgeSlice::default(), 4)),
                                                  (3, 11, (EdgeSlice::default(), 8)),
                                                  (11, 12, (EdgeSlice::default(), 8)),
                                                  (12, 13, (EdgeSlice::default(), 8)),
                                                  (13, 5, (EdgeSlice::default(), 8))]);
            graph.standardize_edges(17, 3, 3);
            assert_eq!(graph.edge_count(), 13);
            for i in 0..13 {
                if [3, 4].contains(&i) {
                    assert_eq!(graph.edge_weight(EdgeIndex::new(i)).unwrap().1, 2);
                }
                else {
                    assert_eq!(graph.edge_weight(EdgeIndex::new(i)).unwrap().1, 1);
                }
            }
        }

        #[test]
        fn standardizes_contigs_from_example() {
            let mut graph = PtGraph::from_edges(&[(0, 1, (EdgeSlice::default(), 3)),
                                                  (1, 2, (EdgeSlice::default(), 2)),
                                                  (2, 3, (EdgeSlice::default(), 3)),
                                                  (3, 4, (EdgeSlice::default(), 1)),
                                                  (4, 5, (EdgeSlice::default(), 3)),
                                                  (5, 6, (EdgeSlice::default(), 5)),
                                                  (6, 7, (EdgeSlice::default(), 5)),
                                                  (7, 8, (EdgeSlice::default(), 3)),
                                                  (7, 9, (EdgeSlice::default(), 4)),
                                                  (9, 10, (EdgeSlice::default(), 6)),
                                                  (3, 11, (EdgeSlice::default(), 1)),
                                                  (11, 12, (EdgeSlice::default(), 2)),
                                                  (12, 13, (EdgeSlice::default(), 2)),
                                                  (13, 5, (EdgeSlice::default(), 2))]);
            graph.standardize_contigs();
            assert_eq!(graph.edge_count(), 14);
            // assert_eq!(graph.edge_weight(EdgeIndex::new(0)).unwrap().1, 8);
            for i in 0..13 {
                if [0, 1, 2, 7].contains(&i) {
                    assert_eq!(graph.edge_weight(EdgeIndex::new(i)).unwrap().1, 3);
                }
                else if [5, 6, 8, 9].contains(&i) {
                    assert_eq!(graph.edge_weight(EdgeIndex::new(i)).unwrap().1, 5);
                }
                else {
                    assert_eq!(graph.edge_weight(EdgeIndex::new(i)).unwrap().1, 2);
                }
            }
        }
    }
}
