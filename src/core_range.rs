//! Conversions from the `core::range` types, which Rust 1.96 stabilized and a
//! future edition will produce from `a..b` syntax.
//!
//! This module only exists on Rust 1.96 and later (see `build.rs`), so the
//! types it names are allowed despite the older MSRV.
#![allow(clippy::incompatible_msrv)]

use std::range::{Range, RangeFrom, RangeInclusive, RangeToInclusive};

use crate::{
    AnyRange,
    Boundary::{At, Below},
    MultiRange,
};

impl<T: Ord> From<Range<T>> for AnyRange<T> {
    /// `lo..hi` is `[At(lo), Below(hi)]`.
    fn from(r: Range<T>) -> Self {
        AnyRange::new(At(r.start), Below(r.end))
    }
}

impl<T: Ord> From<RangeInclusive<T>> for AnyRange<T> {
    /// `lo..=hi` is `[At(lo), At(hi)]`.
    fn from(r: RangeInclusive<T>) -> Self {
        AnyRange::between(r.start, r.last)
    }
}

impl<T> From<RangeFrom<T>> for AnyRange<T> {
    /// `lo..` is `[At(lo), PosInf]`.
    fn from(r: RangeFrom<T>) -> Self {
        AnyRange::at_least(r.start)
    }
}

impl<T> From<RangeToInclusive<T>> for AnyRange<T> {
    /// `..=hi` is `[NegInf, At(hi)]`.
    fn from(r: RangeToInclusive<T>) -> Self {
        AnyRange::at_most(r.last)
    }
}

impl<T: Ord> From<Range<T>> for MultiRange<T> {
    /// The set holding the same values as the [`AnyRange`] it converts to.
    fn from(r: Range<T>) -> Self {
        AnyRange::from(r).into()
    }
}

impl<T: Ord> From<RangeInclusive<T>> for MultiRange<T> {
    /// The set holding the same values as the [`AnyRange`] it converts to.
    fn from(r: RangeInclusive<T>) -> Self {
        AnyRange::from(r).into()
    }
}

impl<T: Ord> From<RangeFrom<T>> for MultiRange<T> {
    /// The set holding the same values as the [`AnyRange`] it converts to.
    fn from(r: RangeFrom<T>) -> Self {
        AnyRange::from(r).into()
    }
}

impl<T: Ord> From<RangeToInclusive<T>> for MultiRange<T> {
    /// The set holding the same values as the [`AnyRange`] it converts to.
    fn from(r: RangeToInclusive<T>) -> Self {
        AnyRange::from(r).into()
    }
}
