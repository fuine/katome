use ::data::graph::{AmbiguousNodes, EdgeIndex, EdgeWeight, Graph, K_SIZE, NodeIndex,
                    get_ambiguous_nodes, out_degree};
use ::algorithms::pruner::remove_weak_edges;
use ::petgraph::EdgeDirection;

/// Standardize edges of the graph.
pub fn standardize_edges(graph: &mut Graph, g: usize, threshold: EdgeWeight) {
    // calculate sum of all weights of edges (s) and sum of weights lower than threshold (l)
    let (s, l) = graph.raw_edges()
        .iter()
        .fold((0usize, 0usize), |acc, e| {
            if e.weight < threshold {
                (acc.0 + e.weight as usize, acc.1 + e.weight as usize)
            }
            else {
                (acc.0 + e.weight as usize, acc.1)
            }
        });
    let p: f64 = calculate_standardization_ratio(g, K_SIZE, s as usize, l as usize);
    info!("Ratio: {} for g: {} k: {} s: {} l: {}", p, g, K_SIZE, s, l);
    // normalize edges across the graph
    for weight in graph.edge_weights_mut() {
        let new_weight = (*weight as f64 * p) as EdgeWeight;
        // debug!("Old: {} New: {}", *weight, new_weight);
        *weight = if new_weight == 0 && *weight >= threshold {
            1
        }
        else {
            new_weight
        }
    }
    // remove edges with weight 0
    remove_weak_edges(graph, 1);
}

pub type Contig = Vec<EdgeIndex>;
pub type Contigs = Vec<Contig>;

/// Standardize each contig
pub fn standardize_contigs(graph: &mut Graph) {
    // find all ambiguous nodes
    let ambiguous_nodes = get_ambiguous_nodes(graph);
    for node in &ambiguous_nodes {
        // get all contigs for the given node
        let contigs = get_contigs_from_node(graph, *node, &ambiguous_nodes);
        for contig in contigs {
            // standardize_contig
            standardize_contig(graph, contig);
        }
    }
}

pub fn get_contigs_from_node(graph: &Graph, starting_node: NodeIndex,
    ambiguous_nodes: &AmbiguousNodes)
                             -> Contigs {
    let mut contigs = vec![];
    for node in graph.neighbors_directed(starting_node, EdgeDirection::Outgoing) {
        let mut contig = vec![];
        let mut current_node = node;
        let mut current_edge = graph.find_edge(starting_node, node)
            .expect("This should never fail");
        loop {
            let out_degree = out_degree(graph, current_node);
            if out_degree != 1 || ambiguous_nodes.contains(&current_node) {
                break;
            }
            contig.push(current_edge);
            current_edge = graph.first_edge(current_node, EdgeDirection::Outgoing).unwrap();
            current_node = graph.edge_endpoints(current_edge).unwrap().1;
        }
        contigs.push(contig);
    }
    contigs
}

/// Set weights of consecutive `Edge`s in the `Contig` to the mean value
pub fn standardize_contig(graph: &mut Graph, contig: Contig) {
    // sum all weights in the contig
    let sum: usize = contig.iter()
        .map(|&e| *graph.edge_weight(e).expect("Contig disappeared from Graph!") as usize)
        .sum();
    // calculate new, standardizes weight
    let standardized_weight = (sum as f64 / contig.len() as f64) as EdgeWeight;
    // modify all edges in the contig with new value
    for edge in contig {
        *graph.edge_weight_mut(edge).expect("Contig disappeared from Graph!") = standardized_weight;
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
    use super::calculate_standardization_ratio;

    #[test]
    fn standardization_ratio() {
        let p = calculate_standardization_ratio(10, 0, 10, 0);
        assert_eq!(p, 1.0);
    }
}
