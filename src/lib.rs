#![no_std]

#![feature(unboxed_closures)]
#![feature(no_std)]
#![feature(core)]

extern crate core;

pub use sort::{sort, sort_by, insertion_sort, heapsort};

mod sort;
/*
mod std {
    mod slice {
        pub use core::convert::AsRef;
    }
}*/
