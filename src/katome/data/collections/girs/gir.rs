//! Graph's Intermediate Representation (GIR) is used as a middle step during creation of the
//! graph. It deals with data of unknown size better, because it uses only one underlying
//! collection, optimized for efficient memory usage as opposed to ease of use
//! and algorithmic efficiency
// use algorithms::pruner::Clean;
use algorithms::builder::Build;
use data::statistics::HasStats;

// pub trait GIR: Build + Clean {  }
pub trait GIR: Build + HasStats {  }
pub trait Convert<T> {
    fn create_from(T) -> Self;
}