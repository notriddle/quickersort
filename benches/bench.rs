#![feature(slicing_syntax)]
#![feature(macro_rules)]
#![feature(unboxed_closures)]

extern crate introsort;
extern crate test;

use introsort::{heapsort, insertion_sort, sort};
use std::rand::{weak_rng, Rng};
use std::mem;
use test::Bencher;

type BigSortable = (u64,u64,u64,u64);

macro_rules! bench_random(
    ($name: ident, $sortfun: ident, $typ: ty, $n: expr) => (
        #[bench]
        fn $name(b: &mut Bencher) {
            let mut rng = weak_rng();
            b.iter(|| {
                let mut v = rng.gen_iter::<$typ>().take($n).collect::<Vec<$typ>>();
                $sortfun(v[mut]);
            });
            b.bytes = $n * mem::size_of::<$typ>() as u64;
        }
    )
)

bench_random!(sort_tiny_random_small, sort, u8, 5)
bench_random!(sort_tiny_random_medium, sort, u8, 100)
bench_random!(sort_tiny_random_large, sort, u8, 10_000)

bench_random!(sort_random_small, sort, u64, 5)
bench_random!(sort_random_medium, sort, u64, 100)
bench_random!(sort_random_large, sort, u64, 10_000)

bench_random!(sort_big_random_small, sort, BigSortable, 5)
bench_random!(sort_big_random_medium, sort, BigSortable, 100)
bench_random!(sort_big_random_large, sort, BigSortable, 10_000)

#[bench]
fn sort_sorted(b: &mut Bencher) {
    let mut v = Vec::from_fn(10000, |i| i);
    b.iter(|| {
        sort(v[mut]);
    });
    b.bytes = (v.len() * mem::size_of_val(&v[0])) as u64;
}

#[bench]
fn sort_big_sorted(b: &mut Bencher) {
    let mut v = Vec::from_fn(10000u, |i| (i, i, i, i));
    b.iter(|| {
        sort(v[mut]);
    });
    b.bytes = (v.len() * mem::size_of_val(&v[0])) as u64;
}

#[bench]
fn sort_few_unique(b: &mut Bencher) {
    let mut v = Vec::new();
    for i in range(0u32, 10) {
        for _ in range(0u, 100) {
            v.push(i);
        }
    }
    let mut rng = weak_rng();
    b.iter(||{
        rng.shuffle(v[mut]);
        sort(v[mut]);
    });
    b.bytes = (v.len() * mem::size_of_val(&v[0])) as u64;
}

#[bench]
fn sort_equals(b: &mut Bencher) {
    let mut v = Vec::from_elem(1000, 1u);
    b.iter(|| {
        sort(v[mut]);
    });
    b.bytes = (v.len() * mem::size_of_val(&v[0])) as u64;
}

#[bench]
fn sort_huge(b: &mut Bencher) {
    let mut rng = weak_rng();
    let n = 100_000;
    let mut v = rng.gen_iter::<int>().take(n).collect::<Vec<int>>();
    b.iter(|| {
        rng.shuffle(v[mut]);
        sort(v[mut]);
    });
    b.bytes = (n * mem::size_of::<int>()) as u64;
}

#[bench]
fn sort_partially_sorted(b: &mut Bencher) {
    fn partially_sort<T: Ord+::std::fmt::Show>(v: &mut [T]) {
        let s = v.len() / 100;
        if s == 0 { return; }
        let mut sorted = true;
        for c in v.chunks_mut(s) {
            if sorted { sort(c[mut]); }
            sorted = !sorted;
        }
    }
    let mut rng = weak_rng();
    let n = 10_000;
    let mut v = rng.gen_iter::<int>().take(n).collect::<Vec<int>>();
    b.iter(|| {
        rng.shuffle(v[mut]);
        partially_sort(v[mut]);
        sort(v[mut]);
    });
    b.bytes = (n * mem::size_of::<int>()) as u64;
}

#[bench]
fn sort_random_large_heapsort(b: &mut Bencher) {
    let mut rng = weak_rng();
    b.iter(|| {
        let mut v = rng.gen_iter::<u64>().take(10000).collect::<Vec<u64>>();
        heapsort(v[mut], &|a, b| a.cmp(b));
    });
    b.bytes = 10000 * mem::size_of::<u64>() as u64;
}

#[bench]
fn sort_random_medium_insertion_sort(b: &mut Bencher) {
    let mut rng = weak_rng();
    b.iter(|| {
        let mut v = rng.gen_iter::<u64>().take(100).collect::<Vec<u64>>();
        insertion_sort(v[mut], &|a, b| a.cmp(b));
    });
    b.bytes = 100 * mem::size_of::<u64>() as u64;
}

#[bench]
fn sort_random_medium_heapsort(b: &mut Bencher) {
    let mut rng = weak_rng();
    b.iter(|| {
        let mut v = rng.gen_iter::<u64>().take(100).collect::<Vec<u64>>();
        heapsort(v[mut], &|a, b| a.cmp(b));
    });
    b.bytes = 100 * mem::size_of::<u64>() as u64;
}

#[bench]
fn sort_strings(b: &mut Bencher) {
    let mut rng = weak_rng();
    let n = 10_000u;
    let mut v = Vec::with_capacity(n);
    let mut bytes = 0;
    for _ in range(0, n) {
        let len = rng.gen_range(0, 60);
        bytes += len;
        let mut s = String::with_capacity(len);
        if len == 0 {
            v.push(s);
            continue;
        }
        for _ in range(0, len) {
            s.push(rng.gen_range(b'a', b'z') as char);
        }
        v.push(s);
    }

    b.iter(|| {
        rng.shuffle(v[mut]);
        sort(v[mut]);
    });
    b.bytes = bytes as u64;
}
