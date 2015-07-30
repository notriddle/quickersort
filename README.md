# quickersort #

[![Build Status](https://travis-ci.org/notriddle/quickersort.svg?branch=master)](https://travis-ci.org/notriddle/quickersort)

[Documentation](https://www.notriddle.com/rustdoc/quickersort/)

This is an implementation of the introsort sorting algorithm.

Credit where it is due, this is a fork of [veddan/rust-introsort], that works
with stable Rust (and, thus, does not work with `#[no_std]`).

[veddan/rust-introsort]: https://github.com/veddan/rust-introsort

To use with cargo, add the following to your `Cargo.toml`:
```toml
[dependencies]
quickersort = "1.0.0"
```
and in your crate root, add
```rust
extern crate quickersort;
```

## Interface ##
The interface is similar to the standard library `sort` and `sort_by` functions.

An example:
```rust
extern crate quickerosort;

fn main() {
    let mut ss = vec!["Introsort", "or", "introspective", "sort", "is",
                      "a", "hybrid", "sorting", "algorithm", "that",
                      "provides", "both", "fast", "average",
                      "performance", "and", "(asymptotically)", "optimal",
                      "worst-case", "performance"];
    quickersort::sort(&mut ss[..]);
    println!("alphabetically");
    for s in ss.iter() { println!("\t{}", s); }
    quickersort::sort_by(&mut ss[..], &|a, b| a.len().cmp(&b.len()));
    println!("\nby length");
    for s in ss.iter() { println!("\t{}", s); }
}
```

Unlike the standard library sort function, introsort is _not_ a stable sort.

## Details ##
At its heart, it is a dual-pivot quicksort.
For partition with many equal elements, it will instead use a single-pivot quicksort optimized for this case.
It detects excessive recursion during quicksort and switches to heapsort if need be, guaranteeing O(n log(n)) runtime on all inputs.
For small partitions it uses insertion sort instead of quicksort.

Unlike the `std` sort, it does not allocate.

## Performance ##
It is quite fast, outperforming the standard sort on all data sets I have tried.
The performance difference varies depending on the characteristics of the data.
On large, completely random arrays, introsort is only 5-10% faster than the standard sort.
However, introsort's performance is greatly improved if the data has few unique values or is (partially) sorted (including reversed data).
For sorted data, introsort is ~4-5 times faster, and for data with few unique values it can be more than 20 times faster.

[Detailed benchmark data](perf.txt) (only for integers as of now) is available.

## Floating point ##
The crate, if built with the "float" feature (which is the default), also includes a `sort_floats` function.
Floating point numbers are not `Ord`, only `PartialOrd`, so `sort` can not be used on them.
The ordering used by `sort_floats` is
```
| -inf | < 0 | -0 | +0 | > 0 | +inf | NaN |
```
`sort_floats` is much more efficient than passing a comparator function implementing this ordering to `sort_by`.

