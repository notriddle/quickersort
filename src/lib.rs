#![no_std]

#![feature(slicing_syntax)]
#![feature(unboxed_closures)]
#![feature(macro_rules)]
#![feature(globs)]

extern crate core;

pub use sort::{sort, sort_by, insertion_sort, heapsort};

mod sort;

mod std {
    mod slice {
        pub use core::slice::AsSlice;
    }
}