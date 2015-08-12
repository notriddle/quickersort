use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::cmp::{min, max};
use std::mem::{size_of, swap};
use std::ptr;

/// The smallest number of elements that may be quicksorted.
/// Must be at least 9.
const MIN_QUICKSORT_ELEMS: usize = 10;

/// The maximum number of elements to be insertion sorted.
const MAX_INSERTION_SORT_ELEMS: usize = 42;

/// Controls the number of elements to be insertion sorted.
/// Higher values give more insertion sorted elements.
const INSERTION_SORT_FACTOR: usize = 450;

pub fn sort_by<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C) {
    if maybe_insertion_sort(v, compare) { return; }
    let heapsort_depth = (3 * log2(v.len())) / 2;
    do_introsort(v, compare, 0, heapsort_depth);
}

pub fn sort<T: Ord>(v: &mut [T]) {
    sort_by(v, &|a, b| a.cmp(b));
}

fn introsort<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C, rec: u32, heapsort_depth: u32) {
    if maybe_insertion_sort(v, compare) { return; }
    do_introsort(v, compare, rec, heapsort_depth);
}

fn do_introsort<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C, rec: u32, heapsort_depth: u32) {
    macro_rules! maybe_swap(
        ($v: expr, $a: expr, $b: expr, $compare: expr) => {
            if compare_idxs($v, *$a, *$b, $compare) == Greater {
                swap($a, $b);
            }
        }
    );

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

fn maybe_insertion_sort<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C) -> bool {
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

pub fn insertion_sort<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C) {
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

fn dual_pivot_sort<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], pivots: (usize, usize, usize, usize, usize),
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

    introsort(&mut v[..lesser], compare, rec + 1, heapsort_depth);
    introsort(&mut v[lesser+1..greater], compare, rec + 1, heapsort_depth);
    introsort(&mut v[greater+1..], compare, rec + 1, heapsort_depth);
}

fn single_pivot_sort<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], pivot: usize, compare: &C, rec: u32, heapsort_depth: u32) {
    let (l, r) = fat_partition(v, pivot, compare);
    let n = v.len();
    if l > 1 {
        introsort(&mut v[..l], compare, rec + 1, heapsort_depth);
    }
    if r > 1 {
        introsort(&mut v[n - r..], compare, rec + 1, heapsort_depth);
    }
}

/// Partitions elements, using the element at `pivot` as pivot.
/// After partitioning, the array looks as following:
/// <<<<<==>>>
/// Return (number of < elements, number of > elements)
fn fat_partition<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], pivot: usize, compare: &C) -> (usize, usize)  {
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

unsafe fn swap_many<T>(v: &mut [T], a: usize, b: usize, n: usize) {
    let mut i = 0;
    while i < n {
        unsafe_swap(v, a + i, b + i);
        i += 1;
    }
}

#[cold]
#[inline(never)]
pub fn heapsort<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C) {
    let mut end = v.len() as isize;
    heapify(v, compare);
    while end > 0 {
        end -= 1;
        v.swap(0, end as usize);
        siftdown_range(v, 0, end as usize, compare);
    }
}

fn heapify<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C) {
    let mut n = (v.len() as isize).wrapping_sub(1) / 4;
    while n >= 0 {
        siftdown(v, n as usize, compare);
        n -= 1;
    }
}

fn siftup<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], start: usize, mut pos: usize, compare: &C) {
    unsafe {
        let new = ptr::read(v.get_unchecked_mut(pos));

        let mut parent = pos.wrapping_sub(1) / 4;

        while pos > start && compare(&new, v.get_unchecked(parent)) == Greater {
            let x = ptr::read(v.get_unchecked_mut(parent));
            ptr::write(v.get_unchecked_mut(pos), x);
            pos = parent;
            parent = (pos - 1) / 4;
        }
        ptr::write(v.get_unchecked_mut(pos), new);
    }
}

fn siftdown_range<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], mut pos: usize, end: usize, compare: &C) {
    unsafe {
        let start = pos;
        let new = ptr::read(v.get_unchecked_mut(pos));

        let mut m_left = 4 * pos + 2;
        while m_left < end {
            let left = m_left - 1;
            let m_right = m_left + 1;
            let right = m_left + 2;
            let largest_left = if compare_idxs(v, left, m_left, compare) == Less {
                m_left
            } else {
                left
            };
            let largest_right = if right < end && compare_idxs(v, m_right, right, compare) == Less {
                right
            } else {
                m_right
            };
            let child = if m_right < end && compare_idxs(v, largest_left, largest_right, compare) == Less {
                largest_right
            } else {
                largest_left
            };
            let x = ptr::read(v.get_unchecked_mut(child));
            ptr::write(v.get_unchecked_mut(pos), x);
            pos = child;
            m_left = 4 * pos + 2;
        }
        let left = m_left - 1;
        if left < end {
            let x = ptr::read(v.get_unchecked_mut(left));
            ptr::write(v.get_unchecked_mut(pos), x);
            pos = left;
        }

        ptr::write(v.get_unchecked_mut(pos), new);
        siftup(v, start, pos, compare);
    }
}

fn siftdown<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], pos: usize, compare: &C) {
    let len = v.len();
    siftdown_range(v, pos, len, compare);
}

fn log2(x: usize) -> u32 {
    if x <= 1 { return 0; }
    let n = x.leading_zeros();
    size_of::<usize>() as u32 * 8 - n
}

#[inline(always)]
unsafe fn compare_idxs<T, C: Fn(&T, &T) -> Ordering>(v: &[T], a: usize, b: usize, compare: &C) -> Ordering {
    let x = v.get_unchecked(a);
    let y = v.get_unchecked(b);
    compare(x, y)
}

#[inline(always)]
fn compare_idxs_safe<T, C: Fn(&T, &T) -> Ordering>(v: &[T], a: usize, b: usize, compare: &C) -> Ordering {
    compare(&v[a], &v[b])
}

#[inline(always)]
unsafe fn unsafe_swap<T>(v: &mut[T], a: usize, b: usize) {
    ptr::swap(v.get_unchecked_mut(a) as *mut T, v.get_unchecked_mut(b) as *mut T);
}
