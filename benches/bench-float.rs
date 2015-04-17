#![feature(test)]
#![feature(unboxed_closures)]

extern crate introsort;
extern crate test;
extern crate rand;

#[cfg(feature = "float")]
mod bench {

use std::f64;
use introsort::sort_floats;
use rand::{Rng, weak_rng};

#[bench]
fn bench_float_f64_large(bench: &mut ::test::Bencher) {
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

    bench.iter(|| {
        let mut w = v.clone();
        sort_floats(&mut w[..]);
    });
    bench.bytes = (::std::mem::size_of::<f64>() * v.len()) as u64;
}

}