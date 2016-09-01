//! Basic type declarations used throughout katome
use std::sync::{Arc, RwLock};

pub type Idx = usize;
pub type EdgeWeight = u16;
pub const K_SIZE: Idx = 40;
pub const WEAK_EDGE_THRESHOLD: EdgeWeight = 4;

pub type Sequences = Vec<u8>;
pub type VecArc = Arc<RwLock<Sequences>>;
