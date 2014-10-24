#![feature(slicing_syntax)]

extern crate test;

use std::cmp::min;
use std::mem::{size_of, zeroed, replace};
use std::ptr;

/// For up to this many small elements, insertion sort will be used
const INSERTION_SMALL_THRESHOLD: uint = 32;

/// For up to this many big elements, insertion sort will be used
const INSERTION_LARGE_THRESHOLD: uint = 16;

/// Element size in bytes from which a element is considered "large" for the purposes
/// of insertion sort threshold selection;
const LARGE_ELEM_THRESHOLD: uint = 16;

/// For more than this many elements (but fewer than `MEDIAN_MEDIAN_THRESHOLD`) the pivot
/// selection is done by median of 3. For fewer elements, the middle one is chosen.
const MEDIAN_THRESHOLD: uint = 64;

/// For more than this many elements, median of 3 median-of-3 will be used for pivot selection.
const MEDIAN_MEDIAN_THRESHOLD: uint = 256;

pub fn sort<T>(v: &mut [T], mut compare: |&T, &T| -> Ordering) {
    if maybe_insertion_sort(v, &mut compare) { return; }
    let heapsort_depth = (3 * log2(v.len())) / 2;
    do_introsort(v, &mut compare, 0, heapsort_depth);
}

fn introsort<T>(v: &mut [T], compare: &mut |&T, &T| -> Ordering, rec: u32, heapsort_depth: u32) {
    if maybe_insertion_sort(v, compare) { return; }
    do_introsort(v, compare, rec, heapsort_depth);
}

fn do_introsort<T>(v: &mut [T], compare: &mut |&T, &T| -> Ordering, rec: u32, heapsort_depth: u32) {
    if rec > heapsort_depth {
        heapsort(v, compare);
        return;
    }

    let pivot = find_pivot(v, compare);
    let (l, r) = partition(v, pivot, compare);
    let n = v.len();
    if l > 0 { introsort(v[mut ..l], compare, rec + 1, heapsort_depth); }
    if r > 0 { introsort(v[mut n - r..], compare, rec + 1, heapsort_depth); }
}

fn maybe_insertion_sort<T>(v: &mut [T], compare: &mut |&T, &T| -> Ordering) -> bool {
    let n = v.len();
    if n <= 1 {
        return true;
    }

    if (size_of::<T>() >= LARGE_ELEM_THRESHOLD && n <= INSERTION_LARGE_THRESHOLD)
            || n <= INSERTION_SMALL_THRESHOLD {
        insertion_sort(v, compare);
        return true;
    }
    return false;
}

fn insertion_sort<T>(v: &mut [T], compare: &mut |&T, &T| -> Ordering) {
    let mut i = 1;
    let n = v.len();
    while i < n {
        let mut j = i;
        while j > 0 && unsafe { (*compare)(v.unsafe_get(j-1), v.unsafe_get(j)) } == Greater {
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
            let r = unsafe { (*compare)(v.unsafe_get(b), v.unsafe_get(0)) };
            if r == Greater { break; }
            if r == Equal {
                v.swap(a, b);
                a += 1;
            }
            b += 1;
        }
        while c >= b {
            let r = unsafe { (*compare)(v.unsafe_get(c), v.unsafe_get(0)) };
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

fn heapsort<T>(v: &mut [T], compare: &mut |&T, &T| -> Ordering) {
    heapify(v, compare);
    let mut end = v.len();
    while end > 0 {
        end -= 1;
        v.swap(0, end);
        siftdown_range(v, 0, end, compare);
    }
}

fn heapify<T>(v: &mut [T], compare: &mut |&T, &T| -> Ordering) {
    let mut n = v.len() / 2;
    while n > 0 {
        n -= 1;
        siftdown(v, n, compare)
    }
}

fn siftup<T>(v: &mut [T], start: uint, mut pos: uint, compare: &mut |&T, &T| -> Ordering) {
    unsafe {
        let new = replace(&mut v[pos], zeroed());

        while pos > start {
            let parent = (pos - 1) >> 1;
            if (*compare)(&new, &v[parent]) == Greater {
                let x = replace(&mut v[parent], zeroed());
                ptr::write(&mut v[pos], x);
                pos = parent;
                continue
            }
            break
        }
        ptr::write(&mut v[pos], new);
    }
}

fn siftdown_range<T>(v: &mut [T], mut pos: uint, end: uint, compare: &mut |&T, &T| -> Ordering) {
    unsafe {
        let start = pos;
        let new = replace(&mut v[pos], zeroed());

        let mut child = 2 * pos + 1;
        while child < end {
            let right = child + 1;
            if right < end && (*compare)(&v[child], &v[right]) != Greater {
                child = right;
            }
            let x = replace(&mut v[child], zeroed());
            ptr::write(&mut v[pos], x);
            pos = child;
            child = 2 * pos + 1;
        }

        ptr::write(&mut v[pos], new);
        siftup(v, start, pos, compare);
    }
}

fn siftdown<T>(v: &mut [T], pos: uint, compare: &mut |&T, &T| -> Ordering) {
    let len = v.len();
    siftdown_range(v, pos, len, compare);
}

fn log2(v: uint) -> u32 {
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

    // TODO Replace with some intrinsic
    if v == 0 { return 0; }
    if size_of::<uint>() == 8 {
        log2_64(v as u64)
    } else {
        log2_32(v as u32)
    }
}

#[cfg(test)]
mod test_sort {
    use super::{introsort, partition, insertion_sort, heapsort, log2};
    use std::rand::{Rng, weak_rng};

    fn introsort_wrapper<T>(v: &mut [T], compare: &mut |&T, &T| -> Ordering) {
       let heapsort_depth = (3 * log2(v.len())) / 2;
       introsort(v, compare, 0, heapsort_depth);
    }

    #[test]
    fn test_introsort() {
        do_test_sort(&introsort_wrapper::<uint>);
    }

    #[test]
    fn test_heapsort() {
        do_test_sort(&heapsort::<uint>);
    }

    #[test]
    fn test_insertion_sort() {
        do_test_sort(&insertion_sort::<uint>);
    }

    fn do_test_sort(sort: &fn(&mut [uint], &mut |&uint, &uint| -> Ordering)) {
        let mut cmp = |a: &uint, b: &uint| a.cmp(b);
        let mut cmp_rev = |a: &uint, b: &uint| b.cmp(a);
        for len in range(4u, 25) {
            for _ in range(0i, 100) {
                let mut v = weak_rng().gen_iter::<u8>().take(len).map(|x| x as uint)
                                        .collect::<Vec<uint>>();
                let mut v1 = v.clone();

                println!("{}", v);
                (*sort)(v[mut], &mut cmp);
                println!("{}", v);
                assert!(v.as_slice().windows(2).all(|w| w[0] <= w[1]));

                (*sort)(v1[mut], &mut cmp);
                assert!(v1.as_slice().windows(2).all(|w| w[0] <= w[1]));

                (*sort)(v1[mut], &mut cmp_rev);
                assert!(v1.as_slice().windows(2).all(|w| w[0] >= w[1]));
            }
        }

        // shouldn't fail/crash
        let mut v: [uint, .. 0] = [];
        (*sort)(v[mut], &mut cmp);

        let mut v = [0xDEADBEEFu];
        (*sort)(v[mut], &mut cmp);
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
    use super::{heapsort, insertion_sort, sort};
    use std::rand::{weak_rng, Rng};
    use std::mem;
    use test::Bencher;

    #[bench]
    fn sort_random_small(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(5).collect::<Vec<u64>>();
            sort(v[mut], cmp);
        });
        b.bytes = 5 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_random_medium(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(100).collect::<Vec<u64>>();
            sort(v[mut], cmp);
        });
        b.bytes = 100 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_random_large(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(10000).collect::<Vec<u64>>();
            sort(v[mut], cmp);
        });
        b.bytes = 10000 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_sorted(b: &mut Bencher) {
        let mut v = Vec::from_fn(10000, |i| i);
        b.iter(|| {
            sort(v[mut], cmp);
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
            sort(v[mut], cmp);
        });
        b.bytes = 5 * mem::size_of::<BigSortable>() as u64;
    }

    #[bench]
    fn sort_big_random_medium(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<BigSortable>().take(100)
                           .collect::<Vec<BigSortable>>();
            sort(v[mut], cmp);
        });
        b.bytes = 100 * mem::size_of::<BigSortable>() as u64;
    }

    #[bench]
    fn sort_big_random_large(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<BigSortable>().take(10000)
                           .collect::<Vec<BigSortable>>();
            sort(v[mut], cmp);
        });
        b.bytes = 10000 * mem::size_of::<BigSortable>() as u64;
    }

    #[bench]
    fn sort_big_sorted(b: &mut Bencher) {
        let mut v = Vec::from_fn(10000u, |i| (i, i, i, i));
        b.iter(|| {
            sort(v[mut], cmp);
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
            sort(v[mut], cmp);
        });
        b.bytes = (v.len() * mem::size_of_val(&v[0])) as u64;
    }

    #[bench]
    fn sort_equals(b: &mut Bencher) {
        let mut v = Vec::from_elem(1000, 1u);
        b.iter(|| {
            sort(v[mut], cmp);
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
            sort(v[mut], cmp);
        });
        b.bytes = (n * mem::size_of::<int>()) as u64;
    }

    #[bench]
    fn sort_partially_sorted(b: &mut Bencher) {
        fn partially_sort<T: Ord>(v: &mut [T]) {
            let s = v.len() / 100;
            if s == 0 { return; }
            let mut sorted = true;
            for c in v.chunks_mut(s) {
                if sorted { sort(c[mut], cmp); }
                sorted = !sorted;
            }
        }
        let mut rng = weak_rng();
        let n = 10_000;
        let mut v = rng.gen_iter::<int>().take(n).collect::<Vec<int>>();
        b.iter(|| {
            rng.shuffle(v[mut]);
            partially_sort(v[mut]);
            sort(v[mut], cmp);
        });
        b.bytes = (n * mem::size_of::<int>()) as u64;
    }

    #[bench]
    fn sort_random_large_heapsort(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(10000).collect::<Vec<u64>>();
            heapsort(v[mut], &mut |a: &u64, b| a.cmp(b));
        });
        b.bytes = 10000 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_random_medium_insertion_sort(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(100).collect::<Vec<u64>>();
            insertion_sort(v[mut], &mut |a: &u64, b| a.cmp(b));
        });
        b.bytes = 100 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_random_medium_heapsort(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(100).collect::<Vec<u64>>();
            heapsort(v[mut], &mut |a: &u64, b| a.cmp(b));
        });
        b.bytes = 100 * mem::size_of::<u64>() as u64;
    }

    fn cmp<T: Ord>(a: &T, b: &T) -> Ordering { a.cmp(b) }
}
