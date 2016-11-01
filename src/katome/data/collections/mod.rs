//! Collection types.
//!
//! Collections are split into two basic subgroups: `GIR`s and `Graph`s.
pub mod girs;
pub mod graphs;
// reexport basic collections and traits

pub use self::girs::GIR;
pub use self::girs::hm_gir::HmGIR;
pub use self::girs::hs_gir::HsGIR;
pub use self::graphs::Graph;
pub use self::graphs::pt_graph::PtGraph;
