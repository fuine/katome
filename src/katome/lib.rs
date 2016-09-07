#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]

#[macro_use]
extern crate log;
extern crate pbr;
#[macro_use]
extern crate lazy_static;

extern crate petgraph;

#[macro_use]
extern crate unwrap;


pub mod data;
pub mod algorithms;
pub mod asm;
