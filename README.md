# rangecomp

[![Crates.io](https://img.shields.io/crates/v/rangecomp.svg)](https://crates.io/crates/rangecomp)
[![Documentation](https://docs.rs/rangecomp/badge.svg)](https://docs.rs/rangecomp)
[![CI](https://github.com/developmentseed/rangecomp/actions/workflows/ci.yml/badge.svg)](https://github.com/developmentseed/rangecomp/actions/workflows/ci.yml)

Set operations on ranges of any ordered type. A `MultiRange` holds a set of
values as sorted, non-overlapping ranges, such as `(-inf, 0) ∪ {5} ∪ [10, 20)`,
with each end exactly open, closed or unbounded. You can take unions,
intersections, differences and complements, test membership and subsets, and
compare individual ranges.

```toml
[dependencies]
rangecomp = "0.1"
```

```rust
use rangecomp::{AnyRange, MultiRange};

let small = MultiRange::from(..10);
let picked = MultiRange::from([3, 7, 12].map(AnyRange::point));

assert_eq!((&small & &picked).points(), Some(vec![&3, &7]));
assert_eq!((&small | &picked).to_string(), "(-inf, 10) ∪ {12}");
assert!((!&small).contains(&10));
```

There are no dependencies unless you enable an optional feature.

## How ranges work

Each end of a range is a [`Boundary`]. A value `v` has three boundaries, just
below it, at it, and just above it, and there are two infinities:

```text
NegInf < Below(5) < At(5) < Above(5) < Below(6) < ... < PosInf
```

Open and closed ends are ordinary points on this line, so `x < 5` is
`[NegInf, Below(5)]`, `x <= 5` is `[NegInf, At(5)]`, `x > 5` is
`[Above(5), PosInf]` and `x = 5` is `[At(5), At(5)]`. Comparing and merging
ranges never needs a special case for open versus closed ends.

An [`AnyRange`] is one such range. A [`MultiRange`] is a set of them, kept
sorted, with overlapping or touching ranges merged, so each set has exactly
one representation.

## Building ranges and sets

`AnyRange` has a constructor for each comparison, and converts from Rust's
range syntax, from a pair of [`Bound`]s, and from a pair of `Boundary`s:

| Condition | `AnyRange` |
|---|---|
| `x = v` | `point(v)` |
| `x < v` | `less_than(v)`, `..v` |
| `x <= v` | `at_most(v)`, `..=v` |
| `x > v` | `greater_than(v)`, `(Excluded(v), Unbounded)` |
| `x >= v` | `at_least(v)`, `v..` |
| `lo <= x <= hi` | `between(lo, hi)`, `lo..=hi` |
| `lo <= x < hi` | `lo..hi` |
| `lo < x <= hi` | `(Excluded(lo), Included(hi))`, `new(Above(lo), At(hi))` |
| everything | `full()`, `..` |
| nothing | `empty()` |

Anything that converts to an `AnyRange` also converts to a `MultiRange`, and
so does an array of them. Sets can also be collected from an iterator of
`AnyRange`s, or grown with `insert` and `extend`. On Rust 1.96 and later the
`core::range` types convert as well.

```rust
use std::ops::Bound::{Excluded, Included, Unbounded};
use rangecomp::{AnyRange, Boundary::{Above, At, Below}, MultiRange};

// Ranges
assert_eq!(AnyRange::from(1..5), AnyRange::new(At(1), Below(5)));
assert_eq!(AnyRange::from(1..=5), AnyRange::between(1, 5));
assert_eq!(AnyRange::from(1..), AnyRange::at_least(1));
assert_eq!(AnyRange::from(..5), AnyRange::less_than(5));
assert_eq!(AnyRange::from(..=5), AnyRange::at_most(5));
assert_eq!(AnyRange::<i32>::from(..), AnyRange::full());
assert_eq!(AnyRange::from((Excluded(1), Included(5))), AnyRange::new(Above(1), At(5)));
assert_eq!(AnyRange::from((Excluded(1), Unbounded)), AnyRange::greater_than(1));
assert_eq!(AnyRange::from((Above(1), At(5))), AnyRange::new(Above(1), At(5)));
assert!(AnyRange::from(5..1).is_empty());

// Sets
let a = MultiRange::from(1..5);
let b = MultiRange::from(AnyRange::point(7));
let c = MultiRange::from((Excluded(1), Unbounded));
let d = MultiRange::from([7..9, 1..4, 3..5]);
let e = MultiRange::from([AnyRange::from(..0), AnyRange::from(10..)]);
let f: MultiRange<i32> = (0..10).step_by(3).map(AnyRange::point).collect();

assert_eq!(d.to_string(), "[1, 5) ∪ [7, 9)");
assert_eq!(e, !MultiRange::from(0..10));
assert_eq!(f.to_string(), "{0} ∪ {3} ∪ {6} ∪ {9}");

let mut g = MultiRange::from(1..3);
g.insert(AnyRange::point(3));
g.extend([AnyRange::from(3..5), AnyRange::from(10..20)]);
assert_eq!(g, MultiRange::from([1..5, 10..20]));
```

An array holds a single type, so mixed kinds such as `..0` and `10..` need
converting to `AnyRange` first, as in `e` above. To build a large set, collect
or extend it in one go rather than taking unions one at a time.

Values can be any `Ord + Clone` type: integers, `char`, `String`, `&str`,
timestamps, dates, decimals. `f64` is not `Ord`, so wrap it first, for
example in `ordered_float::NotNan`. An infinite float is then an ordinary
value: `x <= f64::INFINITY` is `[NegInf, At(inf)]`, not the full range.

## Set operations

| Method | Operator | Values |
|---|---|---|
| `union` | `a \| b` | in either set |
| `intersection` | `a & b` | in both |
| `difference` | `a - b` | in `a` but not `b` |
| `symmetric_difference` | `a ^ b` | in exactly one |
| `complement` | `!a` | not in `a` |

The operators take two owned sets or two references. To inspect a set there
are `contains`, `is_empty`, `is_full`, `is_subset`, `is_superset`,
`is_disjoint`, `hull` (the smallest single range covering the set) and
`points` (the values, if the set is only single values).

```rust
use rangecomp::{AnyRange, MultiRange};

let a = MultiRange::from(0..10);
let b = MultiRange::from(5..15);
assert_eq!(&a ^ &b, MultiRange::from([0..5, 10..15]));
assert!((&a - &b).is_subset(&a));
assert!((MultiRange::from(..0) & MultiRange::from(10..)).is_empty());
assert_eq!((!MultiRange::from(AnyRange::point(5))).to_string(), "(-inf, 5) ∪ (5, inf)");
```

`Display` writes interval notation. To convert a set into another
representation, iterate over its ranges and match on `lower()` and `upper()`.
A lower end is always `NegInf`, `At` or `Above` and an upper end is always
`Below`, `At` or `PosInf`. `to_bounds()` gives the same information as std
`Bound`s.

## Comparing ranges

`AnyRange` implements the thirteen relations of
[Allen's interval algebra](https://en.wikipedia.org/wiki/Allen%27s_interval_algebra):
`before`, `meets`, `overlaps`, `starts`, `during`, `finishes`, `equals`, and
their inverses `after`, `met_by`, `overlapped_by`, `started_by`, `encloses`
and `finished_by`. `intersects` and `disjoint` are there too.

Ends are compared as boundaries, so open ends count:

```rust
use rangecomp::AnyRange;

assert!(AnyRange::between(1, 5).meets(&AnyRange::between(5, 9)));
assert!(AnyRange::from(1..5).meets(&AnyRange::between(5, 9)));
assert!(AnyRange::from(1..5).before(&AnyRange::greater_than(5)));
```

## Integers and other discrete types

The set operations treat values as points on a continuous line: between 4
and 5 there is always room for more. For integers that means the range
`(4, 5)` is not empty, and the union of `..=4` and `5..` is not full.
Membership is still exact, so `contains` is never wrong, but equality and
checks like `is_empty` see the gaps.

Types with a next and previous value implement [`Discrete`]: the integers,
`char`, `bool`, and with the `jiff` or `chrono` feature, calendar dates. For
them, `discrete()` closes open ends (`x > 4` becomes `x >= 5`), merges ranges
with nothing between them, and makes the type's first and last values
unbounded. Compare discrete forms when you need integer semantics. `values()`
lists the values.

```rust
use rangecomp::{AnyRange, MultiRange};

let gap = MultiRange::from(AnyRange::greater_than(4)) & MultiRange::from(..5);
assert!(!gap.is_empty());
assert!(gap.discrete().is_empty());

let split = MultiRange::from([AnyRange::at_most(4), AnyRange::at_least(5)]);
assert!(split.discrete().is_full());

let few = MultiRange::from(AnyRange::greater_than(0)) & MultiRange::from(..4);
assert_eq!(few.values().unwrap().collect::<Vec<i32>>(), [1, 2, 3]);
```

## Features

- `jiff`: `Discrete` for `jiff::civil::Date`
- `chrono`: `Discrete` for `chrono::NaiveDate`
- `serde`: `Serialize` and `Deserialize` for all three types. A set is a list
  of `{"lower": .., "upper": ..}` ranges, and is sorted and merged again when
  read back.

The minimum supported Rust version is 1.70.

## License

MIT

[`Boundary`]: https://docs.rs/rangecomp/latest/rangecomp/enum.Boundary.html
[`AnyRange`]: https://docs.rs/rangecomp/latest/rangecomp/struct.AnyRange.html
[`MultiRange`]: https://docs.rs/rangecomp/latest/rangecomp/struct.MultiRange.html
[`Discrete`]: https://docs.rs/rangecomp/latest/rangecomp/trait.Discrete.html
[`Bound`]: https://doc.rust-lang.org/std/ops/enum.Bound.html
