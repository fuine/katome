//! Algorithms for standardization of edges/contigs in the `Graph`.

use algorithms::pruner::Clean;
use collections::Graph;
use collections::graphs::pt_graph::{EdgeIndex, NodeIndex, PtGraph};
use prelude::{EdgeWeight, K_SIZE};
use petgraph::EdgeDirection;

/// Contig representation.
type Contig = Vec<EdgeIndex>;
/// Vector of `Contig`s.
type GraphContigs = Vec<Contig>;

/// Trait describing standardization of the `Graph`.
pub trait Standardizable {
    /// Standardize edges of the `Graph`.
    fn standardize_edges(&mut self, original_genome_length: usize, threshold: EdgeWeight);
    /// Standardize each contig.
    fn standardize_contigs(&mut self);
}

impl Standardizable for PtGraph {
    fn standardize_edges(&mut self, original_genome_length: usize, threshold: EdgeWeight) {
        // calculate sum of all weights of edges (s) and sum of weights lower than threshold (l)
        let (s, l) = self.raw_edges()
            .iter()
            .fold((0usize, 0usize), |acc, e| {
                if e.weight.1 < threshold {
                    (acc.0 + e.weight.1 as usize, acc.1 + e.weight.1 as usize)
                }
                else {
                    (acc.0 + e.weight.1 as usize, acc.1)
                }
            });
        let p: f64 =
            calculate_standardization_ratio(original_genome_length, K_SIZE, s as usize, l as usize);
        info!("Ratio: {} for g: {} k: {} s: {} l: {}", p, original_genome_length, K_SIZE, s, l);
        // normalize edges across the graph
        for weight in self.edge_weights_mut() {
            let new_weight = (weight.1 as f64 * p).round() as EdgeWeight;
            // debug!("Old: {} New: {}", *weight, new_weight);
            weight.1 = if new_weight == 0 && weight.1 >= threshold {
                1
            }
            else {
                new_weight
            }
        }
        // remove edges with weight 0
        self.remove_weak_edges(1);
    }

    fn standardize_contigs(&mut self) {
        let ambiguous_nodes = self.get_ambiguous_nodes();
        debug!("I found {} ambiguous nodes", ambiguous_nodes.len());
        for node in &ambiguous_nodes {
            let contigs = get_contigs_from_node(self, *node, &ambiguous_nodes);
            for contig in contigs {
                standardize_contig(self, contig);
            }
        }
    }
}

fn get_contigs_from_node(graph: &PtGraph, starting_node: NodeIndex,
                         ambiguous_nodes: &<PtGraph as Graph>::AmbiguousNodes)
                         -> GraphContigs {
    let mut contigs = vec![];
    for node in graph.neighbors_directed(starting_node, EdgeDirection::Outgoing) {
        let mut contig = vec![];
        let mut current_node = node;
        let mut current_edge = unwrap!(graph.find_edge(starting_node, node));
        loop {
            let out_degree = graph.out_degree(&current_node);
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
    pub use ::data::slices::EdgeSlice;
    pub use super::*;
    use super::calculate_standardization_ratio;

    #[test]
    fn calculates_standardization_ratio() {
        let p = calculate_standardization_ratio(10, 0, 10, 0);
        assert_eq!(p, 1.0);
    }

    describe! standardize_contig {
        it "standardizes contigs in empty graph" {
            let mut g = PtGraph::default();
            assert_eq!(g.node_count(), 0);
            assert_eq!(g.edge_count(), 0);
            g.standardize_contigs();
            assert_eq!(g.node_count(), 0);
            assert_eq!(g.edge_count(), 0);
        }

        it "standardizes single contig" {
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

        it "standardizes contig one in two out" {
            let mut graph = PtGraph::from_edges(&[
                (0, 1, (EdgeSlice::default(), 8)), (1, 2, (EdgeSlice::default(), 4)),
                (2, 3, (EdgeSlice::default(), 115)), (3, 4, (EdgeSlice::default(), 1)),
                (2, 5, (EdgeSlice::default(), 2)), (5, 6, (EdgeSlice::default(), 4)),
                (6, 7, (EdgeSlice::default(), 9))
            ]);
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

        it "standardizes contigs two in one out" {
            let mut graph = PtGraph::from_edges(&[
                (0, 1, (EdgeSlice::default(), 8)), (1, 2, (EdgeSlice::default(), 4)),
                (2, 3, (EdgeSlice::default(), 115)), (3, 4, (EdgeSlice::default(), 1)),
                (7, 2, (EdgeSlice::default(), 2)), (5, 6, (EdgeSlice::default(), 4)),
                (6, 7, (EdgeSlice::default(), 9))
            ]);
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

        it "standardizes contigs two in two out" {
            let mut graph = PtGraph::from_edges(&[
                (0, 1, (EdgeSlice::default(), 8)), (1, 2, (EdgeSlice::default(), 4)),
                (2, 3, (EdgeSlice::default(), 115)), (3, 4, (EdgeSlice::default(), 1)),
                (7, 2, (EdgeSlice::default(), 2)), (5, 6, (EdgeSlice::default(), 4)),
                (6, 7, (EdgeSlice::default(), 9)), (2, 8, (EdgeSlice::default(), 178)),
                (8, 9, (EdgeSlice::default(), 298)), (9, 10, (EdgeSlice::default(), 123)),
                (10, 11, (EdgeSlice::default(), 9128))
            ]);
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

        it "standardizes contigs in cycle" {
            let mut graph = PtGraph::from_edges(&[
                (0, 1, (EdgeSlice::default(), 8)), (1, 2, (EdgeSlice::default(), 4)),
                (2, 3, (EdgeSlice::default(), 115)), (3, 1, (EdgeSlice::default(), 1)),
            ]);
            graph.standardize_contigs();
            assert_eq!(graph.edge_count(), 4);
            assert_eq!(graph.edge_weight(EdgeIndex::new(0)).unwrap().1, 8);
            for i in 1..4 {
                assert_eq!(graph.edge_weight(EdgeIndex::new(i)).unwrap().1, 40);
            }
        }
    }
}
