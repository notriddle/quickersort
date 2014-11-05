#![feature(slicing_syntax)]
#![feature(overloaded_calls)]
#![feature(unboxed_closures)]
#![feature(macro_rules)]

extern crate test;

pub use sort::{sort, sort_by};

mod sort;