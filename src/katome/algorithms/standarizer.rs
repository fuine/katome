use ::data::types::{Graph, Weight, K_SIZE};
use ::algorithms::hardener::{remove_weak_edges};

pub fn standarize_edges(graph: &mut Graph, g: usize, n: usize, l: f64) {
    let p: f64 = calculate_standarization_ratio(g, n, l, K_SIZE);
    info!("Ratio: {} for G: {} N: {} L: {} k: {}", p, g, n, l, K_SIZE);
    for edges in graph.values_mut(){
        for edge in edges.outgoing.iter_mut(){
            edge.1 = (edge.1 as f64 * p) as Weight;
        }
    }
    // remove edges with weight 0
    remove_weak_edges(graph, 1);
}

// p = G / (N * (L−k+1) )
// G - original genome length
// N - number of reads
// L - average read length
// k - K_SIZE
#[inline]
fn calculate_standarization_ratio(g: usize, n: usize, l: f64, k: usize) -> f64 {
    g as f64 / (n as f64 * (l - (k as f64) + 1.0))
}

#[cfg(test)]
mod tests {
    use super::{standarize_edges, calculate_standarization_ratio};

    #[test]
    fn standarization_ratio() {
        let p = calculate_standarization_ratio(10, 10, 0.0, 0);
        assert!(p == 1.0);
    }
}
