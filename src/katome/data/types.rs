extern crate fnv;

use std::collections::HashMap as HM;
use std::collections::HashSet;
use std::sync::Arc;
use std::cell::RefCell;
use data::read_slice::{ReadSlice};
use data::edges::{Edges};
use std::hash::BuildHasherDefault;
use self::fnv::FnvHasher;

pub type VertexId = usize;
pub type Weight = u16;
pub const K_SIZE: VertexId = 40;

pub type MyHasher = BuildHasherDefault<FnvHasher>;
pub type Graph = HM<ReadSlice, Edges, MyHasher>;
pub type Sequences = Vec<u8>;
pub type VecArc = Arc<RefCell<Sequences>>;
pub type Nodes = HashSet<VertexId, MyHasher>;

pub struct Asm{
    pub graph: Graph,
    pub in_nodes: Nodes,
    pub out_nodes: Nodes,
}
