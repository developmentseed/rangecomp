# Changelog

All notable changes to this project are documented in this file. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project uses [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-10-22

### Added

- `Boundary<T>`, a totally ordered endpoint type: `NegInf`, `Below(v)`,
  `At(v)`, `Above(v)` and `PosInf`, with `as_ref` and `map`.
- `AnyRange<T>`:
  - constructors `point`, `less_than`, `at_most`, `greater_than`, `at_least`,
    `between`, `empty`, `full` and `new`;
  - conversions from every std range type, from `(Bound<T>, Bound<T>)`, and
    from the `core::range` types on Rust 1.96 and later;
  - `to_bounds`, `as_ref`, `contains`, `intersection` and `discrete`;
  - Allen's interval relations.
- `MultiRange<T>`, a normalized set of ranges:
  - `union`, `intersection`, `difference`, `symmetric_difference` and
    `complement`, also available as the `|`, `&`, `-`, `^` and `!` operators;
  - `contains`, `is_subset`, `is_superset`, `is_disjoint`, `hull`, `points`,
    `insert`, `discrete` and `values`;
  - conversions from ranges, range expressions and arrays, plus
    `FromIterator` and `Extend`.
- `Discrete`, implemented for the integer types, `char` and `bool`, and for
  `jiff::civil::Date` and `chrono::NaiveDate` behind the `jiff` and `chrono`
  features.
- `Display` for `AnyRange` and `MultiRange`, in interval notation such as
  `(-inf, 0) ∪ {5} ∪ [10, 20)`.
- `Serialize` and `Deserialize` for `Boundary`, `AnyRange` and `MultiRange`
  behind the `serde` feature.

[0.1.0]: https://github.com/developmentseed/rangecomp/releases/tag/v0.1.0
