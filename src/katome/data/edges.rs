use data::types::{VertexId, Weight};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Edges{
    pub outgoing: Box<[(VertexId, Weight)]>,
    pub in_num: usize,                        // data is aligned to 8 bytes in this struct
}

impl Edges {
    pub fn new(to: VertexId) -> Edges {
        Edges {
            outgoing: (vec![(to, 1)]).into_boxed_slice(),
            in_num: 0,
        }
    }
    pub fn empty() -> Edges {
        Edges {
            outgoing: Box::new([]),
            in_num: 0,
        }
    }
}

