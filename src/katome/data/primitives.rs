//! Basic type declarations used throughout katome.
use std::sync::{Arc, RwLock};

/// Index type for both nodes and edges in the graph/gir.
pub type Idx = usize;
/// Type for representing weight of the `Edge`.
pub type EdgeWeight = u16;
/// Size of k-mer.
pub const K_SIZE: Idx = 40;
/// Size of substring of k-mer, used for vertex representation in De Bruijn Graph.
pub const K1_SIZE: Idx = K_SIZE - 1;

/// Stores non-repeating k-mers.
pub type Sequences = Vec<u8>;
/// Wrapper around `Sequences`, which allows for `lazy_static` initialization.
pub type VecArc = Arc<RwLock<Sequences>>;
