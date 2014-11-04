#![feature(slicing_syntax)]
#![feature(overloaded_calls)]
#![feature(unboxed_closures)]
#![feature(macro_rules)]

extern crate test;

use std::cmp::min;
use std::mem::{size_of, zeroed, replace, swap};
use std::ptr;

/// For up to this many small elements, insertion sort will be used
const INSERTION_SMALL_THRESHOLD: uint = 10;

/// For up to this many big elements, insertion sort will be used
const INSERTION_LARGE_THRESHOLD: uint = 10;

/// Element size in bytes from which a element is considered "large" for the purposes
/// of insertion sort threshold selection;
const LARGE_ELEM_THRESHOLD: uint = 16;

pub fn sort_by<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C) {
    if maybe_insertion_sort(v, compare) { return; }
    let heapsort_depth = (3 * log2(v.len())) / 2;
    do_introsort(v, compare, 0, heapsort_depth);
}

#[inline]
pub fn sort<T: Ord+std::fmt::Show>(v: &mut [T]) {
    sort_by(v, &|&: a: &T, b| a.cmp(b));
}

fn introsort<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C, rec: u32, heapsort_depth: u32) {
    if maybe_insertion_sort(v, compare) { return; }
    do_introsort(v, compare, rec, heapsort_depth);
}

fn do_introsort<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C, rec: u32, heapsort_depth: u32) {
    // This works around some bugs with unboxed closures
    macro_rules! maybe_swap(
        ($v: expr, $a: expr, $b: expr, $compare: expr) => {
            if compare_idxs_safe($v, *$a, *$b, $compare) == Greater {
                swap($a, $b);
            }
        }
    )

    if rec > heapsort_depth {
        heapsort(v, compare);
        return;
    }

    let n = v.len();

    // Pivot selection algorithm based on Java's DualPivotQuicksort.

    // Fast approximation of n / 7
    let seventh = (n / 8) + (n / 64) + 1;

    // Pick five element evenly spaced around the middle (inclusive) of the slice.
    let mut e3 = n / 2;
    let mut e2 = e3 - seventh;
    let mut e1 = e3 - 2*seventh;
    let mut e4 = e3 + seventh;
    let mut e5 = e3 + 2*seventh;

    // Sort them with a sorting network.
    maybe_swap!(v, &mut e1, &mut e2, compare);
    maybe_swap!(v, &mut e4, &mut e5, compare);
    maybe_swap!(v, &mut e3, &mut e5, compare);
    maybe_swap!(v, &mut e3, &mut e4, compare);
    maybe_swap!(v, &mut e2, &mut e5, compare);
    maybe_swap!(v, &mut e1, &mut e4, compare);
    maybe_swap!(v, &mut e1, &mut e3, compare);
    maybe_swap!(v, &mut e2, &mut e4, compare);
    maybe_swap!(v, &mut e2, &mut e3, compare);

    if compare_idxs_safe(v, e1, e2, compare) != Equal &&
       compare_idxs_safe(v, e2, e3, compare) != Equal &&
       compare_idxs_safe(v, e3, e4, compare) != Equal &&
       compare_idxs_safe(v, e4, e5, compare) != Equal {
        // No consecutive pivot candidates are the same, meaning there is some variaton.
        dual_pivot_sort(v, (e1, e2, e3, e4, e5), compare, rec, heapsort_depth);
    } else {
        // Two consecutive pivots candidates where the same.
        // There are probably many similar elements.
        single_pivot_sort(v, e3, compare, rec, heapsort_depth);
    }
}

fn maybe_insertion_sort<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C) -> bool {
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

fn insertion_sort<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C) {
    let mut i = 1;
    let n = v.len();
    while i < n {
        let mut j = i;
        while j > 0 && unsafe { compare_idxs(v, j-1, j, compare) } == Greater {
            v.swap(j, j-1);
            j -= 1;
        }
        i += 1;
    }
}

fn dual_pivot_sort<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], pivots: (uint, uint, uint, uint, uint),
                                                               compare: &C, rec: u32, heapsort_depth: u32) {
    let (pmin, p1, pmid, p2, pmax) = pivots;
    let n = v.len();

    v.swap(p1, 0);
    v.swap(p2, n - 1);

    let mut lesser = 1;
    let mut greater = n - 2;

    // Skip elements that are already in the correct position
    while compare_idxs_safe(v, lesser, 0, compare) == Less { lesser += 1; }
    while compare_idxs_safe(v, greater, n - 1, compare) == Greater { greater -= 1; }

    let mut k = lesser;
    // XXX We make some unecessary swaps since we can't leave uninitialized values
    // in `v` in case `compare` unwinds.
    'outer: while k <= greater {
        if compare_idxs_safe(v, k, 0, compare) == Less {
            v.swap(k, lesser);
            lesser += 1;
        } else {
            let cmp = compare_idxs_safe(v, k, n - 1, compare);
            if cmp == Greater || cmp == Equal {
                while k < greater && compare_idxs_safe(v, greater, n - 1, compare) == Greater {
                    greater -= 1;
                }
                v.swap(k, greater);
                greater -= 1;
                if compare_idxs_safe(v, k, 0, compare) == Less {
                    v.swap(k, lesser);
                    lesser += 1;
                }
            }
        }
        k += 1;
    }

    lesser -= 1;
    greater += 1;

    // Swap back pivots
    v.swap(0, lesser);
    v.swap(n - 1, greater);

    // Sort left and right
    introsort(v[mut ..lesser], compare, rec + 1, heapsort_depth);
    introsort(v[mut greater+1..], compare, rec + 1, heapsort_depth);

    // TODO do something clever here
    if lesser >= pmin || greater <= pmax {
        // Center partition is small
        introsort(v[mut lesser+1..greater], compare, rec + 1, heapsort_depth);
    } else {
        // Center partition is big
        introsort(v[mut lesser+1..greater], compare, rec + 1, heapsort_depth);
    }
}

fn single_pivot_sort<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], pivot: uint, compare: &C, rec: u32, heapsort_depth: u32) {
    let (l, r) = fat_partition(v, pivot, compare);
    let n = v.len();
    if r <= 1 {
        introsort(v[mut ..l], compare, rec + 1, heapsort_depth);
    } else if l <= 1 {
        introsort(v[mut n - r..], compare, rec + 1, heapsort_depth);
    } else if r < l {
        introsort(v[mut n - r..], compare, rec + 1, heapsort_depth);
        introsort(v[mut ..l], compare, rec + 1, heapsort_depth);
    } else {
        introsort(v[mut ..l], compare, rec + 1, heapsort_depth);
        introsort(v[mut n - r..], compare, rec + 1, heapsort_depth);
    }
}

/// Partitions elements, using the element at `pivot` as pivot.
/// After partitioning, the array looks as following:
/// <<<<<==>>>
/// Return (number of < elements, number of > elements)
fn fat_partition<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], pivot: uint, compare: &C) -> (uint, uint)  {
    let mut a = 0;
    let mut b = a;
    let mut c = v.len() - 1;
    let mut d = c;
    v.swap(0, pivot);
    loop {
        while b <= c {
            let r = unsafe { compare_idxs(v, b, 0, compare) };
            if r == Greater { break; }
            if r == Equal {
                v.swap(a, b);
                a += 1;
            }
            b += 1;
        }
        while c >= b {
            let r = unsafe { compare_idxs(v, c, 0, compare) };
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

#[cold]
fn heapsort<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C) {
    heapify(v, compare);
    let mut end = v.len();
    while end > 0 {
        end -= 1;
        v.swap(0, end);
        siftdown_range(v, 0, end, compare);
    }
}

fn heapify<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C) {
    let mut n = v.len() / 2;
    while n > 0 {
        n -= 1;
        siftdown(v, n, compare)
    }
}

fn siftup<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], start: uint, mut pos: uint, compare: &C) {
    use std::mem::{transmute};

    unsafe {
        let new = replace(&mut v[pos], zeroed());

        while pos > start {
            let parent = (pos - 1) >> 1;
            // TODO: Get rid of transmute when high-rank lifetimes work
            if compare(transmute(&new), transmute(v.unsafe_get(parent))) == Greater {
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

fn siftdown_range<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], mut pos: uint, end: uint, compare: &C) {
    unsafe {
        let start = pos;
        let new = replace(&mut v[pos], zeroed());

        let mut child = 2 * pos + 1;
        while child < end {
            let right = child + 1;
            if right < end && compare_idxs(v, child, right, compare) != Greater {
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

fn siftdown<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], pos: uint, compare: &C) {
    let len = v.len();
    siftdown_range(v, pos, len, compare);
}

fn log2(x: uint) -> u32 {
    if x <= 1 { return 0; }
    let n = if size_of::<uint>() == 8 {
        (unsafe { std::intrinsics::ctlz64(x as u64) }) as u32
    } else {
        unsafe { std::intrinsics::ctlz32(x as u32) }
    };
    size_of::<uint>() as u32 * 8 - n
}

// TODO Replace this function when unboxed closures work properly
// Blocked on https://github.com/rust-lang/rust/issues/17661
#[inline(always)]
unsafe fn compare_idxs<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &[T], a: uint, b: uint, compare: &C) -> Ordering {
    use std::mem::{transmute};

    let x = v.unsafe_get(a);
    let y = v.unsafe_get(b);
    compare(transmute(x), transmute(y))
}

#[inline(always)]
fn compare_idxs_safe<'a, T: 'a+std::fmt::Show, C: Fn<(&'a T, &'a T), Ordering>>(v: &[T], a: uint, b: uint, compare: &C) -> Ordering {
    use std::mem::{transmute};

    unsafe { compare(transmute(&v[a]), transmute(&v[b])) }
}

mod test_sort {
    use super::{sort_by, insertion_sort, heapsort};
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
            sort(v[mut]);
        });
        b.bytes = 5 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_random_medium(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(100).collect::<Vec<u64>>();
            sort(v[mut]);
        });
        b.bytes = 100 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_random_large(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(10000).collect::<Vec<u64>>();
            sort(v[mut]);
        });
        b.bytes = 10000 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_sorted(b: &mut Bencher) {
        let mut v = Vec::from_fn(10000, |i| i);
        b.iter(|| {
            sort(v[mut]);
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
            sort(v[mut]);
        });
        b.bytes = 5 * mem::size_of::<BigSortable>() as u64;
    }

    #[bench]
    fn sort_big_random_medium(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<BigSortable>().take(100)
                           .collect::<Vec<BigSortable>>();
            sort(v[mut]);
        });
        b.bytes = 100 * mem::size_of::<BigSortable>() as u64;
    }

    #[bench]
    fn sort_big_random_large(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<BigSortable>().take(10000)
                           .collect::<Vec<BigSortable>>();
            sort(v[mut]);
        });
        b.bytes = 10000 * mem::size_of::<BigSortable>() as u64;
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
            heapsort(v[mut], &|&: a: &u64, b| a.cmp(b));
        });
        b.bytes = 10000 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_random_medium_insertion_sort(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(100).collect::<Vec<u64>>();
            insertion_sort(v[mut], &|&: a: &u64, b| a.cmp(b));
        });
        b.bytes = 100 * mem::size_of::<u64>() as u64;
    }

    #[bench]
    fn sort_random_medium_heapsort(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut v = rng.gen_iter::<u64>().take(100).collect::<Vec<u64>>();
            heapsort(v[mut], &|&: a: &u64, b| a.cmp(b));
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
}
