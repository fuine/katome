//! Various statistics for contigs.

use algorithms::collapser::SerializedContigs;

/// Statistics for created contigs.
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct ContigsStats {
    ///  A weighted median statistic such that 50% of the entire assembly is
    ///  contained in contigs equal to or larger than this value.
    pub n50: usize,
    /// The smallest number of contigs whose length sum produces N50.
    pub l50: usize,
    ///  A weighted median statistic such that 90% of the entire assembly is
    ///  contained in contigs equal to or larger than this value.
    pub n90: usize,
    ///  Similar to N50, but uses reference genome size as opposed to assembly
    ///  size.
    pub ng50: usize,
}

/// Create stats for contigs.
pub trait HasContigsStats {
    /// Gets stats for contigs.
    fn stats(&self, original_genome_length: usize) -> ContigsStats;
    /// Prints stats for contigs.
    fn print_stats(&self, original_genome_length: usize) {
        print!("{:#?}", self.stats(original_genome_length));
    }
}

impl HasContigsStats for SerializedContigs {
    fn stats(&self, original_genome_length: usize) -> ContigsStats {
        let mut contigs = self.iter()
            .map(|x| x.len())
            .collect::<Vec<usize>>();
        // reverse-sorted order
        contigs.sort();
        let sum: usize = contigs.iter().sum();
        println!("Sum: {} half: {}", sum, sum / 2);
        let n50_ = n_metrics(&contigs, sum / 2);
        let n90_ = n_metrics(&contigs, (0.1 * sum as f64) as usize);
        let ng50_ = n_metrics(&contigs, original_genome_length / 2);
        let l50_ = contigs.iter()
            .rev()
            .enumerate()
            .scan((0, 0), |s, (i, x)| {
                if s.1 >= sum / 2 {
                    None
                }
                else {
                    *s = (i, s.1 + x);
                    Some(*s)
                }
            })
            .last()
            .unwrap()
            .0 + 1;
        ContigsStats {
            n50: n50_,
            l50: l50_,
            n90: n90_,
            ng50: ng50_,
        }
    }
}

fn n_metrics(collection: &[usize], tipping_point: usize) -> usize {
    collection.iter()
        .scan((0, 0), |s, &x| {
            if s.1 >= tipping_point {
                None
            }
            else {
                *s = (x, s.1 + x);
                Some(*s)
            }
        })
        .last()
        .unwrap()
        .0
}


#[cfg(test)]
mod tests {
    pub use algorithms::collapser::SerializedContigs;
    pub use super::*;
    pub use std::iter::repeat;

    describe! cont_stats {
        it "checks basic stats" {
            let correct_stats = ContigsStats {
                n50: 7,
                l50: 3,
                n90: 3,
                ng50: 7,
            };
            let vec1 = vec![2,3,4,5,6,7,8,9,10];
            let original_length = vec1.iter().sum();
            let mut conts: SerializedContigs = Vec::new();
            for i in vec1 {
                conts.push(repeat("a").take(i).collect::<String>());
            }
            assert_eq!(correct_stats, conts.stats(original_length));
        }

        it "checks example from wiki" {
            let correct_stats_a = ContigsStats {
                n50: 70,
                l50: 2,
                n90: 30,
                ng50: 70,
            };
            let correct_stats_b = ContigsStats {
                n50: 50,
                l50: 3,
                n90: 20,
                ng50: 50,
            };
            let a = vec![80,70,50,40,30,20];
            let b = vec![80,70,50,40,30,20,10,5];
            let original_length_a = a.iter().sum();
            let original_length_b = b.iter().sum();
            let mut conts_a: SerializedContigs = Vec::new();
            let mut conts_b: SerializedContigs = Vec::new();
            for i in a {
                conts_a.push(repeat("a").take(i).collect::<String>());
            }
            for i in b {
                conts_b.push(repeat("b").take(i).collect::<String>());
            }
            assert_eq!(correct_stats_a, conts_a.stats(original_length_a));
            assert_eq!(correct_stats_b, conts_b.stats(original_length_b));
        }
    }
}
