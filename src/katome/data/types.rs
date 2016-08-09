use ::petgraph;

use std::sync::{Arc, RwLock};
use data::read_slice::ReadSlice;

pub type VertexId = usize;
pub type EdgeWeight = u16;
pub const K_SIZE: VertexId = 40;
pub const WEAK_EDGE_THRESHOLD: EdgeWeight = 4;

pub type Sequences = Vec<u8>;
pub type VecArc = Arc<RwLock<Sequences>>;
pub type Graph = petgraph::Graph<ReadSlice, EdgeWeight, petgraph::Directed, VertexId>;
