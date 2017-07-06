#![feature(const_fn)]
#![feature(plugin)]
// Apparently required for static phf objects (or at least OperationGroup)
#![feature(drop_types_in_const)]

#![plugin(phf_macros)]

extern crate phf;

pub mod base;
/// Parsing and representation of the "level-1" IR
pub mod ir;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
