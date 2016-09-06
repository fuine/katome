//! Graph's Intermediate Representation (GIR) is used as a middle step during creation of the
//! graph. It deals with data of unknown size better, because it uses only one underlying
//! collection, optimized for efficient memory usage as opposed to ease of use
//! and algorithmic efficiency
// use algorithms::pruner::Clean;
use algorithms::builder::Build;
use data::statistics::HasStats;

// pub trait GIR: Build + Clean {  }
pub trait GIR: Build + HasStats {  }

/// Convert GIR to petgraph's Graph implementation. At this stage assembler loses information about
/// already seen sequences (in the sense of reasonable, efficient and repeatable check - one can
/// always use iterator with find, which pessimistically yields complexity of O(n), as opposed to
/// O(1) for hashmap).
pub trait Convert<T> {
    fn create_from(T) -> Self;
}
