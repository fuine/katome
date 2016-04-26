use data::types::{VertexId};

#[allow(dead_code)]
#[derive(Clone)]
pub struct Edges{
    pub outgoing: Vec<(VertexId, u64)>, // data is aligned to ptr in tuple
    pub in_num: u32,                        // data is aligned to 8 bytes in this struct
    pub out_num: u32
    // pub weights: Vec<u32>,
    // pub in_size: u32,
    // pub in_vertices: Vec<ReadSlice>
}

impl Edges {
    pub fn new(to: VertexId) -> Edges {
        Edges {
            outgoing: vec![(to, 1)],
            in_num: 0,
            out_num: 1,
            // weights: vec![1],
            // in_vertices: vec![]
        }
    }
    pub fn empty() -> Edges {
        Edges {
            outgoing: vec![],
            in_num: 0,
            out_num: 0,
            // weights: vec![1],
            // in_vertices: vec![]
        }
    }
}

