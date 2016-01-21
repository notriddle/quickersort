// (C) 2015 Viktor Dahl <pazaconyoman@gmail.com>
// (C) 2015 Michael Howell <michael@notriddle.com>
// This file is licensed under the same terms as Rust itself.

#![feature(test)]

extern crate quickersort;
extern crate test;
extern crate rand;

#[cfg(feature = "float")]
mod bench {

use std::f64;
use std::cmp::Ordering::*;
use rand::{Rng, weak_rng};
use quickersort;

#[bench]
fn bench_float_sort_float_f64_large(bench: &mut ::test::Bencher) {
    let v = f64_large();
    bench.iter(|| {
        let mut w = v.clone();
        quickersort::sort_floats(&mut w[..]);
    });
    bench.bytes = (::std::mem::size_of::<f64>() * v.len()) as u64;
}

#[bench]
fn bench_float_sort_by_f64_large(bench: &mut ::test::Bencher) {
    let v = f64_large();
    bench.iter(|| {
        let mut w = v.clone();
        quickersort::sort_by(&mut w[..], &|&a: &f64, &b: &f64| {
            if a.is_nan() && b.is_nan() {
                Equal
            } else if a.is_nan() {
                Greater
            } else if b.is_nan() {
                Less
            } else if a == 0.0 && b == 0.0 {
                if a.is_sign_negative() && b.is_sign_negative() {
                    Equal
                } else if a.is_sign_negative() {
                    Less
                } else if b.is_sign_negative() {
                    Greater
                } else {
                    Equal
                }
            } else {
                a.partial_cmp(&b).unwrap()
            }
        });
    });
    bench.bytes = (::std::mem::size_of::<f64>() * v.len()) as u64;
}

fn f64_large() -> Vec<f64> {
    let mut rng = weak_rng();
    let mut v = rng.gen_iter::<f64>().take(10_1000).collect::<Vec<_>>();
    for _ in 1..100 {
        v.push(f64::NAN);
        v.push(0.0);
        v.push(-0.0);
        v.push(f64::INFINITY);
        v.push(f64::NEG_INFINITY);
    }
    rng.shuffle(&mut v[..]);
    return v;
}

}
