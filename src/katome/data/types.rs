extern crate fnv;

use std::collections::HashMap as HM;
use std::mem;
use std::sync::Arc;
use std::cell::RefCell;
use data::read_slice::{ReadSlice};
use data::edges::{Edges};
use std::hash::BuildHasherDefault;
use self::fnv::FnvHasher;

// pub type VertexId = *const u8;
pub type VertexId = usize;
pub const K_SIZE: usize = 40;

pub fn memy(vlen: usize, ilen: usize) -> usize {
// pub fn memy(num_of_vertices: usize) -> usize {
    let num_of_vertices = vlen*(ilen-K_SIZE);
    (num_of_vertices * 11 / 10).next_power_of_two() * (mem::size_of::<VertexId>() + mem::size_of::<Edges>() + mem::size_of::<u64>()) / (4 * 1024usize.pow(3))
}



pub type MyHasher = BuildHasherDefault<FnvHasher>;
pub type Graph = HM<ReadSlice, Edges, MyHasher>;
pub type Sequences = Vec<u8>;
pub type VecArc = Arc<RefCell<Sequences>>;
// pub type VecARcPtr = *const VecRc;
