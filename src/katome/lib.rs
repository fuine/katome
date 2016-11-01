//! De novo assembly library.
#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate unwrap;
extern crate petgraph;
extern crate metrohash;
extern crate rustc_serialize;
extern crate fixedbitset;


#[macro_use]
mod utils;
#[macro_use]
pub mod slices;
pub mod algorithms;

pub mod asm;
pub use asm::Assemble;
pub use asm::basic_assembler::BasicAsm;

pub mod config;
pub use config::Config;

pub mod collections;
pub mod compress;
pub mod stats;
pub mod prelude;
