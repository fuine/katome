use std::fmt;
use std::fmt::Display;
use std::iter::repeat;
use data::gir::GIR;
use data::graph::{EdgeWeight, Graph, in_degree, out_degree};
use ::petgraph::EdgeDirection;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Opt<T> {
    Full(T),
    Empty,
}

impl<T> Default for Opt<T> {
    fn default() -> Self {
        Opt::Empty
    }
}

#[derive(Default, Debug)]
pub struct Stats {
    capacity: (usize, Opt<usize>),
    node_count: usize,
    edge_count: usize,
    max_edge_weight: Opt<EdgeWeight>,
    avg_edge_weight: Opt<f64>,
    max_in_degree: Opt<usize>,
    max_out_degree: Opt<usize>,
    avg_out_degree: Opt<f64>,
    incoming_vert_count: Opt<usize>,
    outgoing_vert_count: Opt<usize>,
}

impl Stats {
    pub fn with_counts(node_count_: usize, edge_count_: usize) -> Stats {
        let mut stats = Stats::default();
        stats.node_count = node_count_;
        stats.edge_count = edge_count_;
        stats
    }
}

impl PartialEq for Stats {
    // ignore capacity during comparison
    fn eq(&self, other: &Stats) -> bool {
        self.node_count == other.node_count &&
        self.edge_count == other.edge_count &&
        self.max_edge_weight == other.max_edge_weight &&
        self.avg_edge_weight == other.avg_edge_weight &&
        self.max_in_degree == other.max_in_degree &&
        self.max_out_degree == other.max_out_degree &&
        self.avg_out_degree == other.avg_out_degree &&
        self.incoming_vert_count == other.incoming_vert_count &&
        self.outgoing_vert_count == other.outgoing_vert_count
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

impl Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f,
                      "I have the capacity of {}, {} for {} nodes and {} edges",
                      self.capacity.0, self.capacity.1, self.node_count,
                      self.edge_count));
        try!(writeln!(f, "Max edge weight: {}", self.max_edge_weight));
        try!(writeln!(f, "Avg edge weight: {:.2}", self.avg_edge_weight));
        try!(writeln!(f, "Max in degree: {}", self.max_in_degree));
        try!(writeln!(f, "Max out degree: {}", self.max_out_degree));
        try!(writeln!(f, "Avg out degree: {}", self.avg_out_degree));
        let percentage = |x| {
            match x {
                Opt::Full(c) => Opt::Full((c * 100) as f64 / self.node_count as f64),
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

pub trait HasStats {
    fn stats(&self) -> Stats;
    fn print_stats(&self) {
        print!("{}", self.stats());
    }
}

impl HasStats for Graph {
    fn stats(&self) -> Stats {
        let max_weight = unwrap!(self.raw_edges().iter().map(|ref w| w.weight).max(),
                                 "No weights in the self!");
        let avg_edge_weight_ =
            self.raw_edges().iter().map(|w| w.weight).fold(0usize, |s, w| s + w as usize) as f64 /
            self.edge_count() as f64;
        let max_out_degree_ =
            self.node_indices().map(|n| out_degree(self, n)).max().expect("No nodes in the self!");
        let avg_out_degree_ = (self.node_indices()
            .fold(0usize, |m, n| m + out_degree(self, n))) as f64 /
                              self.node_count() as f64;
        let (node_cap, edge_cap) = self.capacity();
        Stats {
            capacity: (node_cap, Opt::Full(edge_cap)),
            node_count: self.node_count(),
            edge_count: self.edge_count(),
            max_edge_weight: Opt::Full(max_weight),
            avg_edge_weight: Opt::Full(avg_edge_weight_),
            max_in_degree: Opt::Full(self.node_indices()
                .map(|n| in_degree(self, n))
                .max()
                .unwrap()),
            max_out_degree: Opt::Full(max_out_degree_),
            avg_out_degree: Opt::Full(avg_out_degree_),
            incoming_vert_count: Opt::Full(self.externals(EdgeDirection::Incoming).count()),
            outgoing_vert_count: Opt::Full(self.externals(EdgeDirection::Outgoing).count()),
        }
    }
}

impl HasStats for GIR {
    fn stats(&self) -> Stats {
        let edge_count_ = self.iter().map(|ref e| e.edges.outgoing.len()).sum::<usize>();
        Stats {
            capacity: (self.capacity(), Opt::Empty),
            node_count: self.len(),
            edge_count: edge_count_,
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
