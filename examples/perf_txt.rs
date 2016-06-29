// (C) 2015 Michael Howell <michael@notriddle.com>
// This file is licensed under the same terms as Rust itself.

extern crate quickersort;
extern crate rand;
extern crate num_traits;

use std::cmp::min;
use std::fmt::{self, Display, Formatter};
use std::mem::size_of;
use rand::{weak_rng, Rng};
use num_traits::PrimInt;

#[derive(Copy,Clone)]
enum Algorithm {
    Std,
    Quickersort,
}

#[derive(Copy,Clone)]
enum Pattern {
    Sawtooth,
    Rand,
    Stagger,
    Plateau,
    Shuffle,
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Pattern::Sawtooth => "sawtooth",
            Pattern::Rand => "rand",
            Pattern::Stagger => "stagger",
            Pattern::Plateau => "plateau",
            Pattern::Shuffle => "shuffle",
        }.fmt(f)
    }
}

#[derive(Copy,Clone)]
enum Variant {
    Ident,
    Reverse,
    ReverseFront,
    ReverseBack,
    Sorted,
    Dither,
}

impl Display for Variant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Variant::Ident => "ident",
            Variant::Reverse => "reverse",
            Variant::ReverseFront => "reverse_front",
            Variant::ReverseBack => "reverse_back",
            Variant::Sorted => "sorted",
            Variant::Dither => "dither",
        }.fmt(f)
    }
}
fn generate_int(pattern: Pattern, variant: Variant, size: usize, m: usize) -> Vec<i32> {
    let mut rng = weak_rng();
    let mut rng_it = rng.gen_iter::<usize>();
    let mut random = || rng_it.next().unwrap();
    let mut ret_val = Vec::with_capacity(size);
    let (mut j, mut k) = (0, 0);
    for i in 0 .. size {
        ret_val.push(match pattern {
            Pattern::Sawtooth => i % m,
            Pattern::Rand => random(),
            Pattern::Stagger => (i*m + i) % size,
            Pattern::Plateau => min(i, m),
            Pattern::Shuffle => if random() % m == 0 { j+=2; j } else { k += 2; k },
        } as i32);
    }
    match variant {
        Variant::Ident => (),
        Variant::Reverse => ret_val.reverse(),
        Variant::ReverseFront => ret_val[0 .. size / 2].reverse(),
        Variant::ReverseBack => ret_val[size / 2 .. ].reverse(),
        Variant::Sorted => quickersort::sort(&mut ret_val),
        Variant::Dither => for x in &mut ret_val { let k = *x % 5; *x = k },
    }
    return ret_val;
}

fn run_test(algorithm: Algorithm, pattern: Pattern, variant: Variant, size: usize, m: usize) -> f64 {
    const TRIAL_COUNT: u64 = 256;
    let mut time = 0;
    for _ in 0 .. TRIAL_COUNT {
        let mut v = generate_int(pattern, variant, size, m);
        let start = std::time::Instant::now();
        match algorithm {
            Algorithm::Std => v.sort(),
            Algorithm::Quickersort => quickersort::sort(&mut v),
        }
        let elapsed = start.elapsed();
        time += elapsed.as_secs() * 1000_000_000 + elapsed.subsec_nanos() as u64;
    }
    let duration = time / TRIAL_COUNT;
    size as f64 / (duration as f64 / 1000f64)
}

fn main() {
    println!("{: >15}{: >15}{: >15}{: >15}{: >15}{: >15}{: >15}", "size", "m", "pattern", "variant", "quicker", "std", "quicker/std");
    for size_pow in 0 .. 6 {
        let size = 10.pow(1+size_pow);
        for m_pow in 0 .. log2(size) {
            let m = 2.pow(1+m_pow);
            for &pattern in &[Pattern::Sawtooth, Pattern::Rand, Pattern::Stagger, Pattern::Plateau, Pattern::Shuffle] {
                for &variant in &[Variant::Ident, Variant::Reverse, Variant::ReverseFront, Variant::ReverseBack, Variant::Sorted, Variant::Dither] {
                    let throughput_std = run_test(Algorithm::Std, pattern, variant, size, m);
                    let throughput_qs = run_test(Algorithm::Quickersort, pattern, variant, size, m);
                    println!("{: >15}{: >15}{: >15}{: >15}{: >15.1}{: >15.1}{: >15.1}", size, m, pattern, variant, throughput_qs, throughput_std, throughput_qs / throughput_std);
                }
            }
        }
    }
}

fn log2(x: usize) -> u32 {
    if x <= 1 { return 0; }
    let n = x.leading_zeros();
    size_of::<usize>() as u32 * 8 - n
}

