// (C) 2015 Viktor Dahl <pazaconyoman@gmail.com>
// This file is licensed under the same terms as Rust itself.

#![cfg_attr(not(feature="float"), no_std)]
extern crate core;

extern crate unreachable;
extern crate nodrop;
#[cfg(feature  = "float")]
extern crate num_traits;

pub use sort::{sort, sort_by, sort_by_key, insertion_sort, heapsort, capped_dropmerge_sort};
#[cfg(feature = "float")]
pub use float::{sort_floats};

mod sort;
#[cfg(feature = "float")]
mod float;
