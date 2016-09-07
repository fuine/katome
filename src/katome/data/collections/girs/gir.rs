//! Graph's Intermediate Representation (GIR) interface.
// use algorithms::pruner::Clean;
use algorithms::builder::Build;
use data::statistics::HasStats;

// pub trait GIR: Build + Clean {  }
/// Graph's Intermediate Representation (GIR) interface.
pub trait GIR: Build + HasStats {  }

/// Convert `GIR` to `Graph`. After conversion assembler can lose information about
/// already seen sequences (in the sense of reasonable, efficient and repeatable check - one can
/// always use iterator with find, which pessimistically yields complexity of O(n), as opposed to
/// O(1) for `GIR`s). Such loss depends upon implementation of the `Graph`, but
/// usually it's better to drop support for efficient sequence check (see
/// `PtGraph`).
pub trait Convert<T> {
    fn create_from(T) -> Self;
}
