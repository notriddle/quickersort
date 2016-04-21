// (C) 2015 Viktor Dahl <pazaconyoman@gmail.com>
// This file is licensed under the same terms as Rust itself.

extern crate unreachable;
#[cfg(feature  = "float")]
extern crate num_traits;

pub use sort::{sort, sort_by, insertion_sort, heapsort};
#[cfg(feature = "float")]
pub use float::{sort_floats};

mod sort;
#[cfg(feature = "float")]
mod float;
