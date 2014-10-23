#![feature(slicing_syntax)]

extern crate test;

use std::cmp::min;

/// For up to this many elements, insertion sort will be used
const INSERTION_THRESHOLD: uint = 16;

/// For more than this many elements (but fewer than `MEDIAN_MEDIAN_THRESHOLD`) the pivot
/// selection is done by median of 3. For fewer elements, the middle one is chosen.
const MEDIAN_THRESHOLD: uint = 30;

/// For more than this many elements, median of 3 median-of-3 will be used for pivot selection.
const MEDIAN_MEDIAN_THRESHOLD: uint = 180;

fn sort<T: ::std::fmt::Show>(v: &mut [T], mut compare: |&T, &T| -> Ordering) {
    let heapsort_depth = 2 * log2(v.len());
    introsort(v, &mut compare, 0, heapsort_depth);
}

fn introsort<T: ::std::fmt::Show>(v: &mut [T], compare: &mut |&T, &T| -> Ordering, rec: u32,
                                  heapsort_depth: u32) {
    let n = v.len();
    //println!("{}", v);
    if n <= 1 {
        return;
    }

    if n <= INSERTION_THRESHOLD {
        insertion_sort(v, compare);
        return;
    }

    if rec >= heapsort_depth {
        println!("heapsort not implemented, quicksorting {} elements", v.len());
    }

    let pivot = find_pivot(v, compare);
    let (l, r) = partition(v, pivot, compare);
    if l > 0 { introsort(v[mut ..l], compare, rec + 1, heapsort_depth); }
    if r > 0 { introsort(v[mut n - r..], compare, rec + 1, heapsort_depth); }
}

fn insertion_sort<T>(v: &mut [T], compare: &mut |&T, &T| -> Ordering) {
    let mut i = 1;
    let n = v.len();
    while i < n {
        let mut j = i;
        while j > 0 && (*compare)(&v[j-1], &v[j]) == Greater {
            v.swap(j, j-1);
            j -= 1;
        }
        i += 1;
    }
}

fn find_pivot<T>(v: &[T], compare: &mut |&T, &T| -> Ordering) -> uint {
    let n = v.len();
    let mid = n / 2;
    if n < MEDIAN_THRESHOLD {
        mid
    } else if n < MEDIAN_MEDIAN_THRESHOLD {
        median3(v, 0, mid, n - 1, compare)
    } else {
        let end = n - 1;
        let s = n / 8;
        let a = median3(v, 0, s, 2 * s, compare);
        let b = median3(v, mid - s, mid, mid + s, compare);
        let c = median3(v, end - 2*s, end - s, end, compare);
        median3(v, a, b, c, compare)
    }
}

/// Partitions elements, using the element at `pivot` as pivot.
/// After partitioning, the array looks as following:
/// <<<<<==>>>
/// Return (number of < elements, number of > elements)
fn partition<T>(v: &mut [T], pivot: uint, compare: &mut |&T, &T| -> Ordering) -> (uint, uint)  {
    let mut a = 0;
    let mut b = a;
    let mut c = v.len() - 1;
    let mut d = c;
    v.swap(0, pivot);
    loop {
        while b <= c {
            let r = (*compare)(&v[b], &v[0]);
            if r == Greater { break; }
            if r == Equal {
                v.swap(a, b);
                a += 1;
            }
            b += 1;
        }
        while c >= b {
            let r = (*compare)(&v[c], &v[0]);
            if r == Less { break; }
            if r == Equal {
                v.swap(c, d);
                d -= 1;
            }
            c -= 1;
        }
        if b > c { break; }
        v.swap(b, c);
        b += 1;
        c -= 1;
    }

    let n = v.len();
    let l = min(a, b - a);
    swap_many(v, 0, b - l, l);
    let r = min(d - c, n - 1 - d);
    swap_many(v, b, n - r, r);

    return (b - a, d - c);
}

fn swap_many<T>(v: &mut [T], a: uint, b: uint, n: uint) {
    let mut i = 0;
    while i < n {
        v.swap(a + i, b + i);
        i += 1;
    }
}

fn median3<T>(v: &[T], a: uint, b: uint, c: uint, compare: &mut |&T, &T| -> Ordering) -> uint {
    if (*compare)(&v[a], &v[b]) == Less {
        if (*compare)(&v[b], &v[c]) == Less {
            b
        } else {
            if (*compare)(&v[a], &v[c]) == Less {
                c
            } else {
                a
            }
        }
    } else {
        if (*compare)(&v[b], &v[c]) == Greater {
            b
        } else {
            if (*compare)(&v[a], &v[c]) == Greater {
                c
            } else {
                a
            }
        }
    }
}

// From "Bit Twiddling Hacks" by Sean Eron Anderson
fn log2_32(mut v: u32) -> u32 {
    const DE_BRUIJN: &'static [u32] = &[
        0, 9, 1, 10, 13, 21, 2, 29, 11, 14, 16, 18, 22, 25, 3, 30,
        8, 12, 20, 28, 15, 17, 24, 7, 19, 27, 23, 6, 26, 5, 4, 31
    ];

    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    DE_BRUIJN[((v * 0x07C4ACDD) >> 27) as uint]
}

// Based on the same idea, http://stackoverflow.com/a/11398748/616150
fn log2_64(mut v: u64) -> u32 {
    const DE_BRUIJN: &'static [u32] = &[
        63,  0, 58,  1, 59, 47, 53,  2,
        60, 39, 48, 27, 54, 33, 42,  3,
        61, 51, 37, 40, 49, 18, 28, 20,
        55, 30, 34, 11, 43, 14, 22,  4,
        62, 57, 46, 52, 38, 26, 32, 41,
        50, 36, 17, 19, 29, 10, 13, 21,
        56, 45, 25, 31, 35, 16,  9, 12,
        44, 24, 15,  8, 23,  7,  6,  5
    ];

    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v |= v >> 32;
    DE_BRUIJN[(((v - (v >> 1))*0x07EDD5E59A4E28C2) >> 58) as uint]
}


fn log2(v: uint) -> u32 {
    // TODO Replace with some intrinsic
    if ::std::mem::size_of::<uint>() == 8 {
        log2_64(v as u64)
    } else {
        log2_32(v as u32)
    }
}

#[cfg(test)]
mod test_sort {
    use super::{sort, partition};
    use std::rand::{Rng, weak_rng};

    #[test]
    fn test_sort() {
        for len in range(4u, 25) {
            for _ in range(0i, 100) {
                let mut v = weak_rng().gen_iter::<uint>().take(len)
                                        .collect::<Vec<uint>>();
                let mut v1 = v.clone();

                //println!("{}", v);
                sort(v[mut], |a, b| a.cmp(b));
                //println!("{}", v);
                assert!(v.as_slice().windows(2).all(|w| w[0] <= w[1]));

                sort(v1.as_mut_slice(), |a, b| a.cmp(b));
                assert!(v1.as_slice().windows(2).all(|w| w[0] <= w[1]));

                sort(v1.as_mut_slice(), |a, b| b.cmp(a));
                assert!(v1.as_slice().windows(2).all(|w| w[0] >= w[1]));
            }
        }

        // shouldn't fail/crash
        let mut v: [uint, .. 0] = [];
        sort(v.as_mut_slice(), |a, b| a.cmp(b));

        let mut v = [0xDEADBEEFu];
        sort(v.as_mut_slice(), |a, b| a.cmp(b));
        assert!(v == [0xDEADBEEF]);
    }

    #[ignore]
    #[test]
    fn test_partition() {
        let mut rng = weak_rng();
        for _ in range(0i, 100) {
            let len = rng.gen_range(0, 20);
            let mut v = Vec::new();
            for _ in range(0, len) {
                v.push(rng.gen_range(-10, 10));
            }
            let pivot = if len == 0 { 0 } else { rng.gen_range(0, len) };
            do_test_partition(v, pivot);
        }
    }

    fn do_test_partition(mut v: Vec<int>, pivot: uint) {
        let pivot_elem = v[pivot];
        println!("{}, {}", v[], pivot_elem);
        let (l, r) = partition(v[mut], pivot, &mut |a, b| a.cmp(b));
        println!("{}", v[]);
        println!("({}, {})", l, r);

        let mut i = 0;
        let mut less = 0;
        let mut greater = 0;
        loop {
            if v[i] == pivot_elem { break; }
            assert!(v[i] < pivot_elem);
            i += 1;
            less += 1;
        }
        loop {
            if v[i] > pivot_elem { break; }
            assert!(v[i] == pivot_elem);
            i += 1;
        }
        while i < v.len() {
            assert!(v[i] > pivot_elem);
            i += 1;
            greater += 1;
        }

        assert_eq!(l, less);
        assert_eq!(r, greater);
    }
}

#[cfg(test)]
mod bench {
    use super::sort;
    use std::rand::{weak_rng, Rng};
    use std::mem;
    use test::Bencher;

    #[bench]
    fn sort_random_small(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(5).collect::<Vec<u64>>();
            sort(v[mut], |a, b| a.cmp(b));
        });
        b.bytes = 5 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_random_medium(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(100).collect::<Vec<u64>>();
            sort(v[mut], |a, b| a.cmp(b));
        });
        b.bytes = 100 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_random_large(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(10000).collect::<Vec<u64>>();
            sort(v[mut], |a, b| a.cmp(b));
        });
        b.bytes = 10000 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_sorted(b: &mut Bencher) {
        let mut v = Vec::from_fn(10000, |i| i);
        b.iter(|| {
            sort(v[mut], |a, b| a.cmp(b));
        });
        b.bytes = (v.len() * mem::size_of_val(&v[0])) as u64;
    }

    type BigSortable = (u64,u64,u64,u64);

    #[bench]
    fn sort_big_random_small(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<BigSortable>().take(5)
                           .collect::<Vec<BigSortable>>();
            sort(v[mut], |a, b| a.cmp(b));
        });
        b.bytes = 5 * mem::size_of::<BigSortable>() as u64;
    }

    #[bench]
    fn sort_big_random_medium(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<BigSortable>().take(100)
                           .collect::<Vec<BigSortable>>();
            sort(v[mut], |a, b| a.cmp(b));
        });
        b.bytes = 100 * mem::size_of::<BigSortable>() as u64;
    }

    #[bench]
    fn sort_big_random_large(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<BigSortable>().take(10000)
                           .collect::<Vec<BigSortable>>();
            sort(v[mut], |a, b| a.cmp(b));
        });
        b.bytes = 10000 * mem::size_of::<BigSortable>() as u64;
    }

    #[bench]
    fn sort_big_sorted(b: &mut Bencher) {
        let mut v = Vec::from_fn(10000u, |i| (i, i, i, i));
        b.iter(|| {
            sort(v[mut], |a, b| a.cmp(b));
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
            sort(v[mut], |a, b| a.cmp(b));
        });
        b.bytes = (v.len() * mem::size_of_val(&v[0])) as u64;
    }

    #[bench]
    fn sort_equals(b: &mut Bencher) {
        let mut v = Vec::from_elem(1000, 1u);
        b.iter(|| {
            sort(v[mut], |a, b| a.cmp(b));
        });
        b.bytes = (v.len() * mem::size_of_val(&v[0])) as u64;
    }
}