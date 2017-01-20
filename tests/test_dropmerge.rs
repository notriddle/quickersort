extern crate quickersort;
extern crate rand;
extern crate itertools;

use quickersort::{sort, capped_dropmerge_sort};
use rand::{Rng, weak_rng};
use itertools::Itertools;

#[test]
fn test_dropmerge_sort_trivial() {
    let cmp = |a: &u8, b: &u8| a.cmp(b);
    for len in (4usize .. 200).step(5) {
        let mut v: Vec<u8> = (0 .. len as u8).collect();
        assert!(capped_dropmerge_sort(&mut v[..], &cmp));
        assert!(v.windows(2).all(|w| w[0] <= w[1]));
    }
}

fn do_test_dropsort_ooo(ooo: usize) {
    let cmp = |a: &u8, b: &u8| a.cmp(b);
    for _ in 0 .. 100 {
        for len in (10usize .. 200).step(5) {
            let mut v: Vec<u8> = (0 .. len as u8).collect();
            let mut ooo_items: Vec<usize> = weak_rng().gen_iter::<usize>().take(ooo).map(|x| x % len).collect();
            sort(&mut ooo_items[..]);
            for i in ooo_items {
                if i == 0 {
                    v[0] = ::std::u8::MAX;
                } else {
                    v[i] = v[i-1].wrapping_sub(1);
                }
            }
            let succeeded = capped_dropmerge_sort(&mut v[..], &cmp);
            assert!(ooo >= 4 || succeeded);
            if succeeded {
                assert!(v.windows(2).all(|w| w[0] <= w[1]));
            }
        }
    }
}

#[test]
fn test_dropmerge_sort_1() {
    do_test_dropsort_ooo(1);
}

#[test]
fn test_dropmerge_sort_2() {
    do_test_dropsort_ooo(2);
}

#[test]
fn test_dropmerge_sort_3() {
    do_test_dropsort_ooo(3);
}

#[test]
fn test_dropmerge_sort_4() {
    do_test_dropsort_ooo(4);
}

#[test]
fn test_dropmerge_sort_8() {
    do_test_dropsort_ooo(8);
}

#[test]
fn test_dropmerge_sort_9() {
    do_test_dropsort_ooo(9);
}

#[test]
fn test_dropmerge_sort_16() {
    do_test_dropsort_ooo(16);
}
