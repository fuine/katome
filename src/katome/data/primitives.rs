//! Basic type declarations used throughout katome.
extern crate parking_lot;
use self::parking_lot::RwLock;

/// Index type for both nodes and edges in the graph/gir.
pub type Idx = usize;
/// Type for representing weight of the `Edge`.
pub type EdgeWeight = u16;
/// Size of k-mer.
pub const K_SIZE: Idx = 40;
/// Size of substring of k-mer, used for vertex representation in De Bruijn Graph.
pub const K1_SIZE: Idx = K_SIZE - 1;
/// Size of the compressed K1 size, calculated as (`K1_SIZE` / `CHARS_PER_BYTE`).ceil()
pub const COMPRESSED_K1_SIZE: Idx = 10;
/// Stores non-repeating k-mers. Boxed slice at index 0 is always used as a temporary value.
pub type Sequences = Vec<Box<[u8]>>;
/// Wrapper around `Sequences`, which allows for `lazy_static` initialization.
pub type LockedSequences = RwLock<Sequences>;
