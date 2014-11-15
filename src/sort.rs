use core::prelude::*;
use core::cmp::{min, max};
use core::mem::{size_of, zeroed, replace, swap, transmute};
use core::ptr;

/// The smallest number of elements that may be quicksorted.
/// Must be at least 9.
const MIN_QUICKSORT_ELEMS: uint = 10;

/// The maximum number of elements to be insertion sorted.
const MAX_INSERTION_SORT_ELEMS: uint = 42;

/// Controls the number of elements to be insertion sorted.
/// Higher values give more insertion sorted elements.
const INSERTION_SORT_FACTOR: uint = 450;

pub fn sort_by<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C) {
    if maybe_insertion_sort(v, compare) { return; }
    let heapsort_depth = (3 * log2(v.len())) / 2;
    do_introsort(v, compare, 0, heapsort_depth);
}

#[inline]
pub fn sort<T: Ord>(v: &mut [T]) {
    sort_by(v, &|&: a: &T, b| a.cmp(b));
}

fn introsort<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C, rec: u32, heapsort_depth: u32) {
    if maybe_insertion_sort(v, compare) { return; }
    do_introsort(v, compare, rec, heapsort_depth);
}

fn do_introsort<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C, rec: u32, heapsort_depth: u32) {
    // This works around some bugs with unboxed closures
    macro_rules! maybe_swap(
        ($v: expr, $a: expr, $b: expr, $compare: expr) => {
            if compare_idxs($v, *$a, *$b, $compare) == Greater {
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
    unsafe {
        maybe_swap!(v, &mut e1, &mut e2, compare);
        maybe_swap!(v, &mut e4, &mut e5, compare);
        maybe_swap!(v, &mut e3, &mut e5, compare);
        maybe_swap!(v, &mut e3, &mut e4, compare);
        maybe_swap!(v, &mut e2, &mut e5, compare);
        maybe_swap!(v, &mut e1, &mut e4, compare);
        maybe_swap!(v, &mut e1, &mut e3, compare);
        maybe_swap!(v, &mut e2, &mut e4, compare);
        maybe_swap!(v, &mut e2, &mut e3, compare);
    }

    if unsafe { compare_idxs(v, e1, e2, compare) != Equal &&
                compare_idxs(v, e2, e3, compare) != Equal &&
                compare_idxs(v, e3, e4, compare) != Equal &&
                compare_idxs(v, e4, e5, compare) != Equal } {
        // No consecutive pivot candidates are the same, meaning there is some variaton.
        dual_pivot_sort(v, (e1, e2, e3, e4, e5), compare, rec, heapsort_depth);
    } else {
        // Two consecutive pivots candidates where the same.
        // There are probably many similar elements.
        single_pivot_sort(v, e3, compare, rec, heapsort_depth);
    }
}

fn maybe_insertion_sort<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C) -> bool {
    let n = v.len();
    if n <= 1 {
        return true;
    }

    let threshold = min(MAX_INSERTION_SORT_ELEMS,
                        max(MIN_QUICKSORT_ELEMS, INSERTION_SORT_FACTOR / size_of::<T>()));
    if n <= threshold {
        insertion_sort(v, compare);
        return true;
    }
    return false;
}

pub fn insertion_sort<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C) {
    let mut i = 1;
    let n = v.len();
    while i < n {
        let mut j = i;
        while j > 0 && unsafe { compare_idxs(v, j-1, j, compare) } == Greater {
            unsafe { unsafe_swap(v, j, j-1); }
            j -= 1;
        }
        i += 1;
    }
}

fn dual_pivot_sort<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], pivots: (uint, uint, uint, uint, uint),
                                                               compare: &C, rec: u32, heapsort_depth: u32) {
    let (_, p1, _, p2, _) = pivots;
    let n = v.len();

    let lp = 0;
    let rp = n - 1;

    v.swap(p1, lp);
    v.swap(p2, rp);

    let mut lesser = 1;
    let mut greater = n - 2;

    unsafe {
        // Skip elements that are already in the correct position
        while compare_idxs(v, lesser, lp, compare) == Less { lesser += 1; }
        while compare_idxs(v, greater, rp, compare) == Greater { greater -= 1; }

        let mut k = lesser;
        // XXX We make some unecessary swaps since we can't leave uninitialized values
        // in `v` in case `compare` unwinds.
        while k <= greater {
            if compare_idxs(v, k, lp, compare) == Less {
                unsafe_swap(v, k, lesser);
                lesser += 1;
            } else {
                let cmp = compare_idxs(v, k, rp, compare);
                if cmp == Greater || cmp == Equal {
                    while k < greater && compare_idxs(v, greater, rp, compare) == Greater {
                        greater -= 1;
                    }
                    unsafe_swap(v, k, greater);
                    greater -= 1;
                    if compare_idxs(v, k, lp, compare) == Less {
                        unsafe_swap(v, k, lesser);
                        lesser += 1;
                    }
                }
            }
            k += 1;
        }
    }

    lesser -= 1;
    greater += 1;

    // Swap back pivots
    unsafe {
        unsafe_swap(v, lp, lesser);
        unsafe_swap(v, rp, greater);
    }

    introsort(v[mut ..lesser], compare, rec + 1, heapsort_depth);
    introsort(v[mut lesser+1..greater], compare, rec + 1, heapsort_depth);
    introsort(v[mut greater+1..], compare, rec + 1, heapsort_depth);
}

fn single_pivot_sort<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], pivot: uint, compare: &C, rec: u32, heapsort_depth: u32) {
    let (l, r) = fat_partition(v, pivot, compare);
    let n = v.len();
    if l > 1 {
        introsort(v[mut ..l], compare, rec + 1, heapsort_depth);
    }
    if r > 1 {
        introsort(v[mut n - r..], compare, rec + 1, heapsort_depth);
    }
}

/// Partitions elements, using the element at `pivot` as pivot.
/// After partitioning, the array looks as following:
/// <<<<<==>>>
/// Return (number of < elements, number of > elements)
fn fat_partition<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], pivot: uint, compare: &C) -> (uint, uint)  {
    let mut a = 0;
    let mut b = a;
    let mut c = v.len() - 1;
    let mut d = c;
    v.swap(0, pivot);
    loop {
        while b <= c {
            let r = compare_idxs_safe(v, b, 0, compare);
            if r == Greater { break; }
            if r == Equal {
                unsafe { unsafe_swap(v, a, b); }
                a += 1;
            }
            b += 1;
        }
        while c >= b {
            let r = compare_idxs_safe(v, c, 0, compare);
            if r == Less { break; }
            if r == Equal {
                unsafe { unsafe_swap(v, c, d); }
                d -= 1;
            }
            c -= 1;
        }
        if b > c { break; }
        unsafe { unsafe_swap(v, b, c); }
        b += 1;
        c -= 1;
    }

    let n = v.len();
    let l = min(a, b - a);
    unsafe { swap_many(v, 0, b - l, l); }
    let r = min(d - c, n - 1 - d);
    unsafe { swap_many(v, b, n - r, r); }

    return (b - a, d - c);
}

unsafe fn swap_many<T>(v: &mut [T], a: uint, b: uint, n: uint) {
    let mut i = 0;
    while i < n {
        unsafe_swap(v, a + i, b + i);
        i += 1;
    }
}

#[cold]
#[inline(never)]
pub fn heapsort<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C) {
    heapify(v, compare);
    let mut end = v.len();
    while end > 0 {
        end -= 1;
        v.swap(0, end);
        siftdown_range(v, 0, end, compare);
    }
}

#[inline]
fn heapify<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], compare: &C) {
    let mut n = v.len() / 2;
    while n > 0 {
        n -= 1;
        siftdown(v, n, compare)
    }
}

#[inline]
fn siftup<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], start: uint, mut pos: uint, compare: &C) {
    unsafe {
        let new = replace(&mut v[pos], zeroed());

        while pos > start {
            let parent = (pos - 1) >> 1;
            // TODO: Get rid of transmute when high-rank lifetimes work
            if (*compare)(transmute(&new), transmute(v.unsafe_get(parent))) == Greater {
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

#[inline]
fn siftdown_range<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], mut pos: uint, end: uint, compare: &C) {
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

#[inline]
fn siftdown<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &mut [T], pos: uint, compare: &C) {
    let len = v.len();
    siftdown_range(v, pos, len, compare);
}

fn log2(x: uint) -> u32 {
    if x <= 1 { return 0; }
    let n = if size_of::<uint>() == 8 {
        (unsafe { ::core::intrinsics::ctlz64(x as u64) }) as u32
    } else {
        unsafe { ::core::intrinsics::ctlz32(x as u32) }
    };
    size_of::<uint>() as u32 * 8 - n
}

// TODO Replace this function when unboxed closures work properly
// Blocked on https://github.com/rust-lang/rust/issues/17661
#[inline(always)]
unsafe fn compare_idxs<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &[T], a: uint, b: uint, compare: &C) -> Ordering {
    let x = v.unsafe_get(a);
    let y = v.unsafe_get(b);
    (*compare)(transmute(x), transmute(y))
}

#[inline(always)]
fn compare_idxs_safe<'a, T: 'a, C: Fn<(&'a T, &'a T), Ordering>>(v: &[T], a: uint, b: uint, compare: &C) -> Ordering {
    unsafe { (*compare)(transmute(&v[a]), transmute(&v[b])) }
}

#[inline(always)]
unsafe fn unsafe_swap<T>(v: &mut[T], a: uint, b: uint) {
    ptr::swap(v.unsafe_mut(a) as *mut T, v.unsafe_mut(b) as *mut T);
}
