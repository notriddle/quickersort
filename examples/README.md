# A brief description of what perf.txt is showing you

* `size` is exactly what it sounds like it means. At size = 10, we're comparing the insertion sort implementations of quickersort and rust-lang/rust#38192

* `m` is the "constant factor" used by the list generator and transformation.

* `pattern` describes what kind of list should be generated at first. For examples where `size` = 20 and `m` = 5:

  * `sawtooth`: A repeating, incrementing list with period `m`: `[ 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4 ]`
  * `rand`: A series of random numbers (note that `m` is not used): `[ 174485772, 204021123, 71605603, 334131482, 785758972, 574747816, 150801346, 844973720, 876360420, 210798088, 688904552, 975251835, 835778151, 935999844, 786148954, 779096211, 338255767, 826878933, 563001734, 315096245 ]`
  * `stagger`: A classic, terrible pseduo-RNG with a short period: `[ 0, 6, 12, 18, 4, 10, 16, 2, 8, 14, 0, 6, 12, 18, 4, 10, 16, 2, 8, 14 ]`
  * `plateau`: A list that starts out incrementing but becomes repeating at `m`: `[ 0, 1, 2, 3, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5 ]`
  * `shuffle`: The result of applying a single pass of "bridge" shuffling to two incrementing lists (note that this is a lopsided shuffle unless `m` is 2): `[ 0, 0, 2, 4, 4, 4, 6, 8, 12, 8, 14, 8, 12, 16, 14, 16, 18, 18, 20, 20 ]`

* After generating the pattern, we then generate a variant by applying a transformation to the list:
  * `ident`: Don't do any transformation. Note that `rand` / `ident` completely ignores `m`.
  * `reverse`: Turn it backwards. Note that this is semantically a no-op for `rand`, and ignores `m`.
  * `reverse_front`: Turn the first `len / 2` items backwards. Note that this is semantically a no-op for `rand`, and ignores `m`.
  * `reverse_back`: Turn the last `len / 2` items backwards. Note that this is semantically a no-op for `rand`, and ignores `m`.
  * `sorted`: Sort the list. Note that this is a no-op for plateau.
  * `dither`: Take all the items modulus `m`, thus increasing the number of duplicates. For example, this will turn plateau into `[ 0, 1, 2, 3, 4, 0, 0, 0, 0, ... ]`.

* After the two-step process of generating the list, the list is copied, and the copies are sorted with both sorting algorithms. The time is recorded, and the "throughput" is computed using the formula `size / ( time / trial_count )`. Larger throughput is better.

* Finally, the ratio of throughputs is taken. A larger ratio means quickersort did better than standard sort, while a ratio smaller than 1 means standard sort did better.
