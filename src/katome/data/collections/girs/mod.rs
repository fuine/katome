//! Graph's Intermediate Representation (GIR).
//!
//! It is used as a middle step during creation of the graph. It deals with data
//! of unknown size better, because it uses only one underlying collection,
//! optimized for efficient memory usage as opposed to ease of use and
//! algorithmic efficiency
pub mod gir;
pub mod hs_gir;
pub mod hm_gir;
