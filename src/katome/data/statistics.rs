//! Various statistics for `Graph`s and `GIR`s.
use std::fmt;
use std::fmt::Display;
use std::iter::repeat;
use data::collections::girs::hs_gir::HsGIR;
use data::collections::girs::hm_gir::HmGIR;
use data::primitives::EdgeWeight;
use data::collections::graphs::graph::Graph;
use data::collections::graphs::pt_graph::PtGraph;
use ::petgraph::EdgeDirection;

/// Just like `Option`, but allows for custom `fmt::Display` implementation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Opt<T> {
    /// Some object.
    Full(T),
    /// No object.
    Empty,
}

impl<T> Default for Opt<T> {
    fn default() -> Self {
        Opt::Empty
    }
}

/// Counts for nodes and edges.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Counts {
    /// Number of nodes in collection.
    pub node_count: usize,
    /// Number of edges in collection.
    pub edge_count: usize,
}

/// Various statistics which are created based on `Graph`s or `GIR`s.
#[derive(Default, Debug, Copy, Clone)]
pub struct Stats {
    /// Capacity of the collection.
    pub capacity: (usize, Opt<usize>),
    /// Counts of the collection.
    pub counts: Counts,
    /// Biggest edge weight found in collection.
    pub max_edge_weight: Opt<EdgeWeight>,
    /// Average edge weight.
    pub avg_edge_weight: Opt<f64>,
    /// Maximal number of incoming edges for the node.
    pub max_in_degree: Opt<usize>,
    /// Maximal number of outgoing edges from the node.
    pub max_out_degree: Opt<usize>,
    /// Average number of outgoing edges per node.
    pub avg_out_degree: Opt<f64>,
    /// Number of incoming nodes (nodes without any incoming edges).
    pub incoming_vert_count: Opt<usize>,
    /// Number of outgoing nodes (nodes without any outgoing edges).
    pub outgoing_vert_count: Opt<usize>,
}

impl Stats {
    /// Creates `Stats` with supplied counts.
    pub fn with_counts(node_count_: usize, edge_count_: usize) -> Stats {
        let mut stats = Stats::default();
        stats.counts = Counts {
            node_count: node_count_,
            edge_count: edge_count_,
        };
        stats
    }
}

impl PartialEq for Stats {
    // ignore capacity during comparison
    fn eq(&self, other: &Stats) -> bool {
        self.counts == other.counts &&
        self.max_edge_weight == other.max_edge_weight &&
        round(self.avg_edge_weight) == round(other.avg_edge_weight) &&
        self.max_in_degree == other.max_in_degree &&
        self.max_out_degree == other.max_out_degree &&
        round(self.avg_out_degree) == round(other.avg_out_degree) &&
        self.incoming_vert_count == other.incoming_vert_count &&
        self.outgoing_vert_count == other.outgoing_vert_count
    }
}

fn round(x: Opt<f64>) -> Opt<f64> {
    match x {
        Opt::Full(a) => Opt::Full((a * 100.0).round() / 100.0),
        Opt::Empty => Opt::Empty
    }
}

impl<T: Display> Display for Opt<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Opt::Full(ref o) => write!(f, "{:.2}", o),
            Opt::Empty => write!(f, "??"),
        }
    }
}

impl Display for Counts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} nodes and {} edges", self.node_count, self.edge_count)
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "I have the capacity of {}, {} for {}",
                      self.capacity.0, self.capacity.1, self.counts));
        try!(writeln!(f, "Max edge weight: {}", self.max_edge_weight));
        try!(writeln!(f, "Avg edge weight: {:.2}", self.avg_edge_weight));
        try!(writeln!(f, "Max in degree: {}", self.max_in_degree));
        try!(writeln!(f, "Max out degree: {}", self.max_out_degree));
        try!(writeln!(f, "Avg out degree: {}", self.avg_out_degree));
        let percentage = |x| {
            match x {
                Opt::Full(c) => Opt::Full((c * 100) as f64 / self.counts.node_count as f64),
                Opt::Empty => Opt::Empty,
            }
        };
        let in_percentage = percentage(self.incoming_vert_count);
        let out_percentage = percentage(self.outgoing_vert_count);
        try!(writeln!(f,
                      "Incoming vertices count: {} ({:.2}%)",
                      self.incoming_vert_count,
                      in_percentage));
        try!(writeln!(f,
                      "Outgoing vertices count: {} ({:.2}%)",
                      self.outgoing_vert_count,
                      out_percentage));
        writeln!(f, "{}", repeat("*").take(20).collect::<String>())
    }
}

/// Create stats for the collection.
pub trait HasStats {
    /// Gets stats for the collection.
    fn stats(&self) -> Stats;
    /// Prints stats for the collection.
    fn print_stats(&self) {
        print!("{}", self.stats());
    }
}

impl HasStats for PtGraph {
    fn stats(&self) -> Stats {
        let max_weight = unwrap!(self.raw_edges().iter().map(|ref w| w.weight).max(),
                                 "No weights in the self!");
        let avg_edge_weight_ =
            self.raw_edges().iter().map(|w| w.weight).fold(0usize, |s, w| s + w as usize) as f64 /
            self.edge_count() as f64;
        let max_out_degree_ =
            self.node_indices().map(|n| self.out_degree(&n)).max().expect("No nodes in the self!");
        let avg_out_degree_ = (self.node_indices()
            .fold(0usize, |m, n| m + self.out_degree(&n))) as f64 /
                              self.node_count() as f64;
        let (node_cap, edge_cap) = self.capacity();
        Stats {
            capacity: (node_cap, Opt::Full(edge_cap)),
            counts: Counts{
                node_count: self.node_count(),
                edge_count: self.edge_count(),
            },
            max_edge_weight: Opt::Full(max_weight),
            avg_edge_weight: Opt::Full(avg_edge_weight_),
            max_in_degree: Opt::Full(self.node_indices()
                .map(|n| self.in_degree(&n))
                .max()
                .unwrap()),
            max_out_degree: Opt::Full(max_out_degree_),
            avg_out_degree: Opt::Full(avg_out_degree_),
            incoming_vert_count: Opt::Full(self.externals(EdgeDirection::Incoming).count()),
            outgoing_vert_count: Opt::Full(self.externals(EdgeDirection::Outgoing).count()),
        }
    }
}

impl HasStats for HsGIR {
    fn stats(&self) -> Stats {
        let edge_count_ = self.iter().map(|e| e.edges.outgoing.len()).sum::<usize>();
        Stats {
            capacity: (self.capacity(), Opt::Empty),
            counts: Counts {
                node_count: self.len(),
                edge_count: edge_count_,
            },
            max_edge_weight: Opt::Empty,
            avg_edge_weight: Opt::Empty,
            max_in_degree: Opt::Empty,
            max_out_degree: Opt::Empty,
            avg_out_degree: Opt::Empty,
            incoming_vert_count: Opt::Empty,
            outgoing_vert_count: Opt::Empty,
        }
    }
}

impl HasStats for HmGIR {
    fn stats(&self) -> Stats {
        let edge_count_ = self.values().map(|e| e.outgoing.len()).sum::<usize>();
        Stats {
            capacity: (self.capacity(), Opt::Empty),
            counts: Counts {
                node_count: self.len(),
                edge_count: edge_count_,
            },
            max_edge_weight: Opt::Empty,
            avg_edge_weight: Opt::Empty,
            max_in_degree: Opt::Empty,
            max_out_degree: Opt::Empty,
            avg_out_degree: Opt::Empty,
            incoming_vert_count: Opt::Empty,
            outgoing_vert_count: Opt::Empty,
        }
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;
    describe! stats {
        it "compares two Stats" {
            let st = Stats {
                capacity: (1024, Opt::Full(1024)),
                counts: Counts {
                    node_count: 1,
                    edge_count: 1,
                },
                max_edge_weight: Opt::Full(806),
                avg_edge_weight: Opt::Full(2.31),
                max_in_degree: Opt::Full(5),
                max_out_degree: Opt::Full(1),
                avg_out_degree: Opt::Full(1.0),
                incoming_vert_count: Opt::Full(25),
                outgoing_vert_count: Opt::Full(0),
            };
            assert!(st == st);
        }

        it "detects difference in counts" {
            let st = Stats {
                capacity: (1024, Opt::Full(1024)),
                counts: Counts {
                    node_count: 1,
                    edge_count: 1,
                },
                max_edge_weight: Opt::Full(806),
                avg_edge_weight: Opt::Full(2.31),
                max_in_degree: Opt::Full(5),
                max_out_degree: Opt::Full(1),
                avg_out_degree: Opt::Full(1.0),
                incoming_vert_count: Opt::Full(25),
                outgoing_vert_count: Opt::Full(0),
            };
            let mut st1 = st;
            st1.counts.node_count += 1;
            assert!(st != st1);
            st1.counts.node_count -= 1;
            assert!(st == st1);
            st1.incoming_vert_count = Opt::Empty;
            assert!(st != st1);
        }

        it "rounds floats properly" {
            let st = Stats {
                capacity: (1024, Opt::Full(1024)),
                counts: Counts {
                    node_count: 1,
                    edge_count: 1,
                },
                max_edge_weight: Opt::Full(806),
                avg_edge_weight: Opt::Full(2.314),
                max_in_degree: Opt::Full(5),
                max_out_degree: Opt::Full(1),
                avg_out_degree: Opt::Full(1.5),
                incoming_vert_count: Opt::Full(25),
                outgoing_vert_count: Opt::Full(0),
            };
            let mut st1 = st.clone();
            st1.avg_edge_weight = Opt::Full(2.313);
            assert!(st == st1);
            st1.avg_out_degree = Opt::Full(1.51);
            assert!(st != st1);
            st1.avg_out_degree = Opt::Full(1.49);
            assert!(st != st1);
        }
    }
}
