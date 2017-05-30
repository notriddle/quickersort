// (C) 2015 Viktor Dahl <pazaconyoman@gmail.com>
// This file is licensed under the same terms as Rust itself.

#![no_std]

extern crate unreachable;
extern crate nodrop;

pub use sort::{sort, sort_by, sort_by_key, insertion_sort, heapsort};
pub use float::{sort_floats};

mod sort;
mod float;
