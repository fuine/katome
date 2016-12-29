//! Basic type and const values declarations used throughout katome.
extern crate parking_lot;
use self::parking_lot::RwLock;
use compress::CHARS_PER_BYTE;

/// Index type for both nodes and edges in the graph/gir.
pub type Idx = usize;
/// Type for representing weight of the `Edge`.
pub type EdgeWeight = u16;
/// Size of k-mer. Please note that until compile-time environment arguments
/// won't be implemented in Rust we need to use static mut to easily change this
/// constant without recompilation/changing sources.
pub static mut K_SIZE: Idx = 40;
/// Size of substring of k-mer, used for vertex representation in De Bruijn Graph. Should always be `K_SIZE` - 1
pub static mut K1_SIZE: Idx = 39;
/// Size of the compressed K1 size, calculated as (`K1_SIZE` / `CHARS_PER_BYTE`).ceil()
pub static mut COMPRESSED_K1_SIZE: Idx = 10;
/// Stores non-repeating k-mers. Boxed slice at index 0 is always used as a temporary value.
pub type Sequences = Vec<Box<[u8]>>;
/// Wrapper around `Sequences`, which allows for `lazy_static` initialization.
pub type LockedSequences = RwLock<Sequences>;

/// Set global variables `K_SIZE`, `K1_SIZE`, `COMPRESSED_K1_SIZE` using
/// provided `k_size`. This function is not thread safe, and thus it is marked
/// unsafe.
pub unsafe fn set_global_k_sizes(k_size: usize) {
    assert!(k_size > 1);
    K_SIZE = k_size;
    K1_SIZE = k_size - 1;
    COMPRESSED_K1_SIZE = (K1_SIZE as f64 / CHARS_PER_BYTE as f64).ceil() as usize;
    info!("Changed global variables: K_SIZE {} K1_SIZE {} COMPRESSED_K1_SIZE {}",
          K_SIZE,
          K1_SIZE,
          COMPRESSED_K1_SIZE);
}
