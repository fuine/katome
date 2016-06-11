use data::types::{VertexId, K_SIZE, VecArc};
// use asm::assembler::{VECTOR_RC};
use std::cmp;
use std::hash;
use std::str;
use std::option::{Option};

#[derive(Eq, Clone)]
pub struct ReadSlice {
    pub offset: VertexId,
    vec: VecArc,
}

impl ReadSlice {
    pub fn new(vec_: VecArc, offset_: VertexId) -> ReadSlice {
        ReadSlice {
            vec: vec_,
            offset: offset_
        }
    }

    pub fn name(&self) -> String {
        str::from_utf8(&self.vec.borrow()[self.offset as usize..(self.offset+K_SIZE) as usize]).unwrap().to_string()
    }
}

impl hash::Hash for ReadSlice {
    fn hash<H>(&self, state: &mut H)
            where H: hash::Hasher {
        self.name().hash(state)
    }
}

impl cmp::PartialEq for ReadSlice {
    fn eq(&self, other: &ReadSlice) -> bool{
        self.name() == other.name()
    }
}

impl cmp::PartialOrd for ReadSlice {
    fn partial_cmp(&self, other: &ReadSlice) -> Option<cmp::Ordering>{
        self.name().partial_cmp(&other.name())
    }
}

impl cmp::Ord for ReadSlice {
    fn cmp(&self, other: &ReadSlice) -> cmp::Ordering{
        self.name().cmp(&other.name())
    }
}
