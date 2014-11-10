#![feature(slicing_syntax)]
#![feature(macro_rules)]

extern crate introsort;
extern crate test;

use introsort::{sort_by, insertion_sort, heapsort};
use std::rand::{Rng, weak_rng};
use std::iter::range_step;

macro_rules! do_test_sort(
    ($sortfun:ident) => ({
        let cmp = |&: a: &uint, b: &uint| a.cmp(b);
        let cmp_rev = |&: a: &uint, b: &uint| b.cmp(a);
        for len in range_step(4u, 250, 5) {
            for _ in range(0i, 100) {
                let mut v = weak_rng().gen_iter::<u8>().take(len).map(|x| 10 + (x % 89) as uint)
                                        .collect::<Vec<uint>>();
                let mut v1 = v.clone();

                //println!("{}", v);
                $sortfun(v[mut], &cmp);
                println!("sorted:\t{}", v);
                assert!(v.as_slice().windows(2).all(|w| w[0] <= w[1]));

                $sortfun(v1[mut], &cmp);
                assert!(v1.as_slice().windows(2).all(|w| w[0] <= w[1]));

                $sortfun(v1[mut], &cmp_rev);
                assert!(v1.as_slice().windows(2).all(|w| w[0] >= w[1]));
            }
        }
        // shouldn't fail/crash
        let mut v: [uint, .. 0] = [];
        $sortfun(v[mut], &cmp);

        let mut v = [0xDEADBEEFu];
        $sortfun(v[mut], &cmp);
        assert!(v == [0xDEADBEEF]);
    })
)

#[test]
fn test_introsort() {
    do_test_sort!(sort_by);
}

#[test]
fn test_heapsort() {
    do_test_sort!(heapsort);
}

#[test]
fn test_insertion_sort() {
    do_test_sort!(insertion_sort);
}
