#![feature(const_fn)]
#![feature(plugin)]

#![plugin(phf_macros)]

extern crate phf;
extern crate symbol_map;

pub mod base;
/// Parsing and representation of the "level-1" IR
pub mod ir;
