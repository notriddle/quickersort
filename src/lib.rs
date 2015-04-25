#![no_std]

#![feature(unboxed_closures)]
#![feature(no_std)]
#![feature(core)]

extern crate core;

#[cfg(feature  = "float")]
extern crate num;

pub use sort::{sort, sort_by, insertion_sort, heapsort};
#[cfg(feature = "float")]
pub use float::{sort_floats};

mod sort;
#[cfg(feature = "float")]
mod float;
