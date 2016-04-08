use std::collections::HashMap as HM;
use std::cmp;
use std::hash;
use std::slice;
use std::str;
use std::option::{Option};
use std::mem;
use std::usize;

// pub type VertexId = *const u8;
pub type VertexId = *const u8;
pub static K_SIZE: usize = 40;

#[allow(dead_code)]
pub struct Edges{
    pub outgoing: Vec<(ReadSlice, u64)>, // data is aligned to ptr in tuple
    pub in_size: u64,                        // data is aligned to 8 bytes in this struct
    // pub weights: Vec<u32>,
    // pub in_size: u32,
    // pub out_size: u32
    // pub in_vertices: Vec<ReadSlice>
}

impl Edges {
    pub fn new(to: ReadSlice) -> Edges {
        Edges {
            outgoing: vec![(to, 1)],
            // weights: vec![1],
            in_size: 0,
            // out_size: 1,
            // in_vertices: vec![]
        }
    }
}

pub fn memy(vlen: usize, ilen: usize) -> usize {
// pub fn memy(num_of_vertices: usize) -> usize {
    let num_of_vertices = vlen*(ilen-K_SIZE);
    (num_of_vertices * 11 / 10).next_power_of_two() * (mem::size_of::<VertexId>() + mem::size_of::<Edges>() + mem::size_of::<u64>()) / (4 * 1024usize.pow(3))
}


#[derive(Eq, Copy, Clone)]
pub struct ReadSlice {
    ptr: VertexId,
}

impl ReadSlice {
    pub fn new(to: VertexId) -> ReadSlice {
        ReadSlice {
            ptr: to,
        }
    }

    pub fn name(&self) -> String {
        let slice = unsafe {
            slice::from_raw_parts(self.ptr, K_SIZE)
        };
        str::from_utf8(slice).unwrap().to_string()
    }
}

impl hash::Hash for ReadSlice {
    fn hash<H>(&self, state: &mut H)
            where H: hash::Hasher {
        let slice = unsafe {
            slice::from_raw_parts(self.ptr, K_SIZE)
        };
        slice.hash(state)
    }
}

impl cmp::PartialEq for ReadSlice {
    fn eq(&self, other: &ReadSlice) -> bool{
        let slice_1 = unsafe {
            slice::from_raw_parts(self.ptr, K_SIZE)
        };
        let slice_2 = unsafe {
            slice::from_raw_parts(other.ptr, K_SIZE)
        };
        slice_1 == slice_2
    }
}

impl cmp::PartialOrd for ReadSlice {
    fn partial_cmp(&self, other: &ReadSlice) -> Option<cmp::Ordering>{
        let slice_1 = unsafe {
            slice::from_raw_parts(self.ptr, K_SIZE)
        };
        let slice_2 = unsafe {
            slice::from_raw_parts(other.ptr, K_SIZE)
        };
        slice_1.partial_cmp(slice_2)
    }
}

impl cmp::Ord for ReadSlice {
    fn cmp(&self, other: &ReadSlice) -> cmp::Ordering{
        let slice_1 = unsafe {
            slice::from_raw_parts(self.ptr, K_SIZE)
        };
        let slice_2 = unsafe {
            slice::from_raw_parts(other.ptr, K_SIZE)
        };
        slice_1.cmp(slice_2)
    }
}

pub type Graph = HM<ReadSlice, Edges>;
pub type Sequence = Vec<u8>;
pub type ReadPtr  = Box<Sequence>;
pub type Sequences = Vec<ReadPtr>;
