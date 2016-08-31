#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]

#![feature(alloc, heap_api)]
extern crate alloc;
#[macro_use]
extern crate log;
extern crate pbr;
extern crate rand;
#[macro_use]
extern crate lazy_static;

extern crate petgraph;

#[macro_use]
extern crate unwrap;


#[macro_use]
pub mod data;
pub mod algorithms;
pub mod asm;
