use data::types::{VertexId};
use data::read_slice::{ReadSlice};

#[allow(dead_code)]
pub struct Edges{
    pub outgoing: Vec<(VertexId, u64)>, // data is aligned to ptr in tuple
    pub in_size: u64,                        // data is aligned to 8 bytes in this struct
    // pub weights: Vec<u32>,
    // pub in_size: u32,
    // pub out_size: u32
    // pub in_vertices: Vec<ReadSlice>
}

impl Edges {
    pub fn new(to: VertexId) -> Edges {
        Edges {
            outgoing: vec![(to, 1)],
            // weights: vec![1],
            in_size: 0,
            // out_size: 1,
            // in_vertices: vec![]
        }
    }
    pub fn empty() -> Edges {
        Edges {
            outgoing: vec![],
            // weights: vec![1],
            in_size: 1,
            // out_size: 1,
            // in_vertices: vec![]
        }
    }
}

