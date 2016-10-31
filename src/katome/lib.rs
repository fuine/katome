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
pub mod data;
pub mod algorithms;
pub mod asm;
pub mod config;
pub use config::Config;
