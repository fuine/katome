use ::data::types::{Graph, EdgeWeight, K_SIZE};
use ::algorithms::pruner::remove_weak_edges;

pub fn standarize_edges(graph: &mut Graph, g: usize, threshold: EdgeWeight) {
    // calculate sum of all weights of edges (s) and sum of weights lower than threshold (l)
    let (s, l) = graph.raw_edges().iter()
        .fold((0usize, 0usize), |acc, ref e| {
            if e.weight < threshold {
                (acc.0 + e.weight as usize, acc.1 + e.weight as usize)
            }
            else {
                (acc.0 + e.weight as usize, acc.1)
            }
        });
    let p: f64 = calculate_standarization_ratio(g, K_SIZE, s as usize, l as usize);
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

// p = G / (N * (L−k+1) )
// G - original genome length
// k - K_SIZE
// s - sum of all weights of edges
// l - sum of weights lower than threshold
#[inline]
fn calculate_standarization_ratio(original_genome_length: usize, k: usize,
                                  sum_of_all_weights: usize,
                                  weights_lower_than_threshold: usize) -> f64 {
    (original_genome_length - k) as f64 / (sum_of_all_weights - weights_lower_than_threshold) as f64
}

// #[cfg(test)]
// mod tests {
// use super::{standarize_edges, calculate_standarization_ratio};

// #[test]
// fn standarization_ratio() {
// let p = calculate_standarization_ratio(10, 10, 0.0, 0);
// assert!(p == 1.0);
