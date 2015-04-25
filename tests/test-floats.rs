extern crate introsort;
extern crate rand;

#[cfg(feature = "float")]
extern crate num;

#[cfg(feature = "float")]
mod bench {

use std::f64;
use num::traits::{ToPrimitive};
use introsort::{sort_floats};
use rand::{Rng, weak_rng};

#[test]
fn test_sort_floats() {
    let sorted_negs: Vec<_> = (0..10u32).map(|i| -1.0f64 * (11 - i).to_f64().unwrap()).collect();
    let sorted_pos: Vec<_> = (0..10u32).map(|i| (i + 1).to_f64().unwrap()).collect();
    let mut sorted = vec![f64::NEG_INFINITY];
    sorted.extend(sorted_negs);
    sorted.push(-0.0);
    sorted.push(0.0);
    sorted.extend(sorted_pos);
    sorted.push(f64::INFINITY);
    for _ in 0..10 {
        sorted.push(f64::NAN);
    }

    let mut rng = weak_rng();
    for _ in 0..1000 {
        let mut w = sorted.clone();
        rng.shuffle(&mut w[..]);
        sort_floats(&mut w[..]);
        assert_float_eq(&w[..], &sorted[..]);
    }

    let mut xs = [0.0f64, -0.0];
    sort_floats(&mut xs[..1]);
    sort_floats(&mut xs[..2]);
    sort_floats(&mut xs[..]);
}

#[test]
fn test_sort_random_floats() {
    let mut rng = weak_rng();
    for _ in 0..1000 {
        let mut v = rng.gen_iter::<f64>().map(|x| 100.0 * (x - 0.5)).take(20).collect::<Vec<_>>();
        v.push(f64::NAN);
        v.push(0.0);
        v.push(-0.0);
        rng.shuffle(&mut v[..]);
        sort_floats(&mut v[..]);
        assert_floats_sorted(&v[..]);
    }
}

#[test]
fn test_sort_zeros() {
    let mut rng = weak_rng();
    let mut v = rng.gen_iter::<f64>().map(|x| if x > 0.5 { 0.0 } else { -0.0 }).take(50).collect::<Vec<_>>();
    sort_floats(&mut v[..]);
    assert_floats_sorted(&v[..]);
}

#[test]
fn test_sort_few_zeros() {
    let mut v = [0.0, -0.0];
    sort_floats(&mut v[..]);
    assert_floats_sorted(&v[..]);
}

fn assert_float_eq(a: &[f64], b: &[f64]) {
    if a.len() != b.len() {
        panic!("different lengths, {} vs. {}", a.len(), b.len());
    }
    for i in 0..a.len() {
        if !((a[i] == b[i] && a[i].signum() == b[i].signum()) || (a[i].is_nan() && b[i].is_nan())) {
            panic!("\n{:?} !=\n{:?}\n{} != {}", a, b, a[i], b[i]);
        }
    }
}

fn assert_floats_sorted(v: &[f64]) {
    let (not_nans, nans) = match v.iter().position(|x| x.is_nan()) {
        Some(i) => v.split_at(i),
        None    => (v, &[][..])
    };
    assert!(not_nans.windows(2).all(|w| {
        let (a, b) = (w[0], w[1]);
        println!("{:?}, {:?}\t{}, {}", a, b, a.signum(), b.signum());
        a < b || (a == b && !(b.is_sign_negative() && a.is_sign_positive()))
    }));
    assert!(nans.iter().all(|x| x.is_nan()));
}

}