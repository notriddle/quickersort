// (C) 2015 Viktor Dahl <pazaconyoman@gmail.com>
// (C) 2015 Michael Howell <michael@notriddle.com>
// This file is licensed under the same terms as Rust itself.

use nodrop::NoDrop;
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

/// Maximum number of swaps to attempt before falling back
/// on quicksort.
const INSERTION_SORTED_CAP: usize = 8;

/// Sort using a comparison function.
///
/// # Example
///
///     let mut unsorted = [1, 3, 9, 2, 6, 5];
///     let sorted = [9, 6, 5, 3, 2, 1];
///     ::quickersort::sort_by(&mut unsorted, &|a, b| b.cmp(a));
///     assert_eq!(unsorted, sorted);
pub fn sort_by<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C) {
    if maybe_insertion_sort(v, compare) { return; }
    let heapsort_depth = (3 * log2(v.len())) / 2;
    do_introsort(v, compare, 0, heapsort_depth);
}

/// Sort using a conversion function.
///
/// # Example
///
///     #[derive(Debug, Eq, PartialEq)]
///     struct Selector {
///         specificity: u32,
///         source_order: u32,
///     }
///     let mut selectors_scrambled = [
///         Selector{ specificity: 1, source_order: 5 },
///         Selector{ specificity: 1, source_order: 4 },
///         Selector{ specificity: 3, source_order: 1 },
///     ];
///     let selectors_sorted = [
///         Selector{ specificity: 1, source_order: 4 },
///         Selector{ specificity: 1, source_order: 5 },
///         Selector{ specificity: 3, source_order: 1 },
///     ];
///     ::quickersort::sort_by_key(
///         &mut selectors_scrambled,
///         |a| (a.specificity, a.source_order)
///     );
///     assert_eq!(selectors_scrambled, selectors_sorted);
pub fn sort_by_key<T, K: Ord, F: Fn(&T) -> K>(v: &mut [T], key: F) {
    sort_by(v, &|a, b| key(a).cmp(&key(b)));
}

/// Sort using the default comparison function.
pub fn sort<T: Ord>(v: &mut [T]) {
    sort_by(v, &|a, b| a.cmp(b));
}

fn introsort<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C, rec: u32, heapsort_depth: u32) {
    if maybe_insertion_sort(v, compare) { return; }
    do_introsort(v, compare, rec, heapsort_depth);
}

fn do_introsort<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C, rec: u32, heapsort_depth: u32) {
    macro_rules! maybe_swap(
        ($v: expr, $a: expr, $b: expr, $compare: expr, $swapped: ident) => {
            if compare_idxs($v, *$a, *$b, $compare) == Greater {
                swap($a, $b);
                $swapped = true;
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
    let mut swapped = false;

    // Sort them with a sorting network.
    unsafe {
        maybe_swap!(v, &mut e1, &mut e2, compare, swapped);
        maybe_swap!(v, &mut e4, &mut e5, compare, swapped);
        maybe_swap!(v, &mut e3, &mut e5, compare, swapped);
        maybe_swap!(v, &mut e3, &mut e4, compare, swapped);
        maybe_swap!(v, &mut e2, &mut e5, compare, swapped);
        maybe_swap!(v, &mut e1, &mut e4, compare, swapped);
        maybe_swap!(v, &mut e1, &mut e3, compare, swapped);
        maybe_swap!(v, &mut e2, &mut e4, compare, swapped);
        maybe_swap!(v, &mut e2, &mut e3, compare, swapped);
    }

    // If the input appears partially sorted, try an insertion sort.
    if !swapped && capped_insertion_sort(v, compare) {
        return;
    }

    // Dual-pivot quicksort behaves very poorly if both pivots are equal.
    // Use a single-pivot quicksort if they are.
    if unsafe { compare_idxs(v, e2, e4, compare) != Equal } {
        DualPivotSort::dual_pivot_sort(v, (e2, e4), compare, rec, heapsort_depth);
    } else {
        // N.B. If compare() is a well-behaved total order,
        // e3 must be equal to e2 and e4.
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

fn capped_insertion_sort<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C) -> bool {
    let mut i = 1;
    let mut cap = INSERTION_SORTED_CAP;
    let n = v.len();
    while i < n {
        let mut j = i;
        while j > 0 && unsafe { compare_idxs(v, j-1, j, compare) } == Greater {
            unsafe { unsafe_swap(v, j, j-1); }
            cap -= 1;
            j -= 1;
            if cap == 0 {
                return false;
            }
        }
        i += 1;
    }
    true
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

struct DualPivotSort<'a, T: 'a> {
    p1: usize,
    pivot1: NoDrop<T>,
    p2: usize,
    pivot2: NoDrop<T>,
    v: &'a mut [T],
}

impl<'a, T: 'a> DualPivotSort<'a, T> {
    fn dual_pivot_sort<C: Fn(&T, &T) -> Ordering>(v: &mut [T], (p1, p2): (usize, usize),
                                                  compare: &C, rec: u32, heapsort_depth: u32) {
        debug_assert!(v.len() > 9);
        let (left, right) = unsafe {
            if compare_idxs(v, p1, p2, compare) == Greater {
                unsafe_swap(v, p1, p2);
            }
            // Move the leftmost and rightmost list elements into the spots formerly occupied by the pivots.
            // This leaves `v[0]` and `v[n-1]` logically uninitialized.
            // Those gaps get filled back in by `DualPivotSort::Drop`.
            // If `compare` unwinds, we'll put the items in p1 and p2 back into 0 and len()-1,
            // otherwise p1 and p2 will be replaced with the last locations in the leftmost and rightmost
            // partitions, where the pivots will be placed.
            let mut this = DualPivotSort{
                p1: p1,
                pivot1: NoDrop::new(ptr::read(v.get_unchecked(p1))),
                p2: p2,
                pivot2: NoDrop::new(ptr::read(v.get_unchecked(p2))),
                v: v,
            };
            ptr::copy(this.v.get_unchecked(0), this.v.get_unchecked_mut(p1), 1);
            ptr::copy(this.v.get_unchecked(this.v.len() - 1), this.v.get_unchecked_mut(p2), 1);
            // Start partitioning:
            let (mut l, mut r) = (1, this.v.len() - 2);
            while l < this.v.len() - 1 && compare(this.v.get_unchecked(l), &*this.pivot1) == Less { l += 1; }
            while r > 0 && compare(this.v.get_unchecked(r), &*this.pivot2) == Greater { r -= 1; }
            // The invariant has been established, and shall now be maintained.
            let v = &mut *this.v;
            let p1 = &*this.pivot1;
            let p2 = &*this.pivot2;
            let mut m = l;
            while m <= r {
                debug_assert!(l != 0);
                debug_assert!(l <= m);
                debug_assert!(l == m || l < r);
                debug_assert!(r != v.len() - 1);
                debug_assert!(m < v.len() && r < v.len() && l < v.len());
                if cfg!(feature="assert_working_compare") {
                    debug_assert!(l == m || compare(&v[l], p1) != Less);
                    debug_assert!(l == 1 || compare(&v[l-1], p1) != Greater);
                    debug_assert!(l <= 2 || compare(&v[l-2], p1) != Greater);
                    debug_assert!(compare(&v[r], p2) != Greater);
                    debug_assert!(r == v.len() - 2 || compare(&v[r+1], p2) != Less);
                    debug_assert!(r >= v.len() - 3 || compare(&v[r+2], p2) != Less);
                }
                let middle = NoDrop::new(ptr::read(v.get_unchecked(m)));
                let middle = &*middle;
                if compare(middle, p1) == Less {
                    ptr::copy(v.get_unchecked(l), v.get_unchecked_mut(m), 1);
                    ptr::copy(middle, v.get_unchecked_mut(l), 1);
                    l += 1;
                } else if compare(middle, p2) == Greater {
                    if compare(v.get_unchecked(r), p1) == Less {
                        ptr::copy(v.get_unchecked(l), v.get_unchecked_mut(m), 1);
                        ptr::copy(v.get_unchecked(r), v.get_unchecked_mut(l), 1);
                        l += 1;
                    } else {
                        ptr::copy(v.get_unchecked(r), v.get_unchecked_mut(m), 1);
                    }
                    ptr::copy(middle, v.get_unchecked_mut(r), 1);
                    r -= 1;
                    while m <= r && compare(v.get_unchecked(r), p2) == Greater {
                        r -= 1;
                    }
                }
                m += 1;
            }
            this.p1 = l - 1;
            this.p2 = r + 1;
            (l, r)
            // DualPivotSort dropped here
        };
        let left_pivot = left - 1;
        let right_pivot = right + 1;
        debug_assert!(right_pivot > left_pivot);
        if cfg!(feature="assert_working_compare") {
            for item in &v[..left_pivot] {
                debug_assert!(compare(item, &v[left_pivot]) != Greater);
                debug_assert!(compare(item, &v[right_pivot]) != Greater);
            }
            for item in &v[left_pivot..right_pivot] {
                debug_assert!(compare(item, &v[left_pivot]) != Less);
                debug_assert!(compare(item, &v[right_pivot]) != Greater);
            }
            for item in &v[right_pivot..] {
                debug_assert!(compare(item, &v[right_pivot]) != Less);
                debug_assert!(compare(item, &v[left_pivot]) != Less);
            }
            debug_assert!(compare(&v[left_pivot], &v[right_pivot]) == Less);
        }
        introsort(&mut v[..left_pivot], compare, rec + 1, heapsort_depth);
        introsort(&mut v[left_pivot + 1..right_pivot], compare, rec + 1, heapsort_depth);
        introsort(&mut v[right_pivot + 1..], compare, rec + 1, heapsort_depth);
    }
    unsafe fn write_pivots(&mut self) {
        let n = self.v.len();
        ptr::copy(self.v.get_unchecked(self.p1), self.v.get_unchecked_mut(0), 1);
        ptr::copy(&*self.pivot1, self.v.get_unchecked_mut(self.p1), 1);
        ptr::copy(self.v.get_unchecked(self.p2), self.v.get_unchecked_mut(n - 1), 1);
        ptr::copy(&*self.pivot2, self.v.get_unchecked_mut(self.p2), 1);
    }
}

impl<'a, T: 'a> Drop for DualPivotSort<'a, T> {
    fn drop(&mut self) {
        unsafe {
            self.write_pivots();
        }
    }
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
        Siftdown::siftdown_range(v, 0, end as usize, compare);
    }
}

fn heapify<T, C: Fn(&T, &T) -> Ordering>(v: &mut [T], compare: &C) {
    let mut n = (v.len() as isize).wrapping_sub(1) / 4;
    while n >= 0 {
        Siftdown::siftdown(v, n as usize, compare);
        n -= 1;
    }
}

struct Siftup<'a, T: 'a> {
    new: NoDrop<T>,
    v: &'a mut [T],
    pos: usize,
}

impl<'a, T: 'a> Siftup<'a, T> {
    fn siftup<C: Fn(&T, &T) -> Ordering>(v_: &mut [T], start: usize, pos_: usize, compare: &C) {
        unsafe {
            let mut this = Siftup{
                new: NoDrop::new(ptr::read(v_.get_unchecked_mut(pos_))),
                v: v_,
                pos: pos_,
            };
            let mut parent = this.pos.wrapping_sub(1) / 4;
            while this.pos > start && compare(&*this.new, this.v.get_unchecked(parent)) == Greater {
                let x = ptr::read(this.v.get_unchecked_mut(parent));
                ptr::write(this.v.get_unchecked_mut(this.pos), x);
                this.pos = parent;
                parent = this.pos.wrapping_sub(1) / 4;
            }
            // siftup dropped here
        }
    }
}

impl<'a, T: 'a> Drop for Siftup<'a, T> {
    fn drop(&mut self) {
        unsafe {
            ptr::copy(&*self.new, self.v.get_unchecked_mut(self.pos), 1);
        }
    }
}

struct Siftdown<'a, T: 'a> {
    new: NoDrop<T>,
    v: &'a mut [T],
    pos: usize,
}

impl<'a, T: 'a> Siftdown<'a, T> {
    fn siftdown_range<C: Fn(&T, &T) -> Ordering>(v_: &mut [T], pos_: usize, end: usize, compare: &C) {
        let pos = unsafe {
            let mut this = Siftdown{
                new: NoDrop::new(ptr::read(v_.get_unchecked_mut(pos_))),
                v: v_,
                pos: pos_,
            };

            let mut m_left = 4 * this.pos + 2;
            while m_left < end {
                let left = m_left - 1;
                let m_right = m_left + 1;
                let right = m_left + 2;
                let largest_left = if compare_idxs(this.v, left, m_left, compare) == Less {
                    m_left
                } else {
                    left
                };
                let largest_right = if right < end && compare_idxs(this.v, m_right, right, compare) == Less {
                    right
                } else {
                    m_right
                };
                let child = if m_right < end && compare_idxs(this.v, largest_left, largest_right, compare) == Less {
                    largest_right
                } else {
                    largest_left
                };
                let x = ptr::read(this.v.get_unchecked_mut(child));
                ptr::write(this.v.get_unchecked_mut(this.pos), x);
                this.pos = child;
                m_left = 4 * this.pos + 2;
            }
            let left = m_left - 1;
            if left < end {
                let x = ptr::read(this.v.get_unchecked_mut(left));
                ptr::write(this.v.get_unchecked_mut(this.pos), x);
                this.pos = left;
            }

            this.pos
            // this dropped here
        };
        Siftup::siftup(v_, pos_, pos, compare);
    }

    fn siftdown<C: Fn(&T, &T) -> Ordering>(v: &mut [T], pos: usize, compare: &C) {
        let len = v.len();
        Siftdown::siftdown_range(v, pos, len, compare);
    }
}

impl<'a, T: 'a> Drop for Siftdown<'a, T> {
    fn drop(&mut self) {
        unsafe {
            ptr::copy(&*self.new, self.v.get_unchecked_mut(self.pos), 1);
        }
    }
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
