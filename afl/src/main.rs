// This program is released under the same terms as Rust itself.

#![feature(plugin)]

#![plugin(afl_plugin)]

extern crate afl;
extern crate quickersort;

use std::io::{self, Read};
use std::cmp::Ordering::{self, Less, Equal, Greater};
use std::cell::Cell;

fn main() {
    let mut input = io::stdin().bytes().map(|x|x.unwrap_or(b'\0'));
    // Build the list that will be "sorted".
    let mut len = input.next().unwrap_or(0) as u32 | ((input.next().unwrap_or(0) as u32) << 8) | ((input.next().unwrap_or(0) as u32) << 16) | ((input.next().unwrap_or(0) as u32) << 24);
    let mut list: Vec<u8> = Vec::new();
    if len > 1024*1024 { len = 1024*1024 } // cap len so that the list never exceeds 1MiB
    for _ in 0..len {
        list.push(0);
    }
    // The rest of the input is directions on what the comparator should return.
    let directions: Vec<Ordering> = input.map(|x| match x { 0 => Less, 1 => Equal, _ => Greater }).take(1024*1024).collect(); // also cap this at 1MiB
    let pos = Cell::new(0);
    let count = directions.len();
    quickersort::sort_by(&mut list, &move |_, _| { pos.set((pos.get() + 1)); if pos.get() >= count { Equal } else { directions[pos.get()] } });
}
