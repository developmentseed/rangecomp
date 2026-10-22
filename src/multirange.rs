use std::{
    fmt, iter, mem,
    ops::{
        BitAnd, BitOr, BitXor, Bound, Not, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
        RangeToInclusive, Sub,
    },
    slice, vec,
};

use crate::{
    discrete::next,
    AnyRange,
    Boundary::{self, At, NegInf, PosInf},
    Discrete,
};

/// A set of values, stored as sorted, non-overlapping [`AnyRange`]s.
///
/// Empty ranges are dropped and ranges that overlap or touch are merged, so a
/// set has one representation and equal sets compare `==`. Anything that
/// converts to an `AnyRange` converts to a `MultiRange`, as does an array of
/// such things; sets can also be collected from an iterator of ranges. The
/// [crate docs](crate#building-ranges-and-sets) list every conversion.
///
/// ```
/// use rangecomp::{AnyRange, MultiRange};
///
/// let set = MultiRange::from([7..9, 1..4, 3..5]);
/// assert_eq!(set.as_slice(), [AnyRange::from(1..5), AnyRange::from(7..9)]);
///
/// let in_list = MultiRange::from([5, 1, 3].map(AnyRange::point));
/// assert_eq!(in_list.to_string(), "{1} ∪ {3} ∪ {5}");
/// ```
///
/// The set operations are available as methods and as operators on owned
/// values or references: `|` is [`union`](Self::union), `&`
/// [`intersection`](Self::intersection), `-` [`difference`](Self::difference),
/// `^` [`symmetric_difference`](Self::symmetric_difference) and `!`
/// [`complement`](Self::complement).
///
/// ```
/// use rangecomp::MultiRange;
///
/// let lt5 = MultiRange::from(..5);
/// let ge5 = MultiRange::from(5..);
/// assert!((&lt5 | &ge5).is_full());
/// assert!((&lt5 & &ge5).is_empty());
/// assert_eq!(!lt5, ge5);
/// ```
///
/// Values are treated as points on a continuous line. For integers and other
/// [`Discrete`] types, use [`discrete`](Self::discrete) before comparing sets
/// or checking emptiness; see its docs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MultiRange<T>(Vec<AnyRange<T>>);

impl<T> MultiRange<T> {
    /// The set with no values. The same as `MultiRange::default()`.
    ///
    /// ```
    /// use rangecomp::MultiRange;
    ///
    /// let none = MultiRange::<i32>::empty();
    /// assert!(none.is_empty() && !none.contains(&0));
    /// ```
    pub fn empty() -> Self {
        MultiRange(Vec::new())
    }

    /// The set of every value.
    ///
    /// ```
    /// use rangecomp::MultiRange;
    ///
    /// let all = MultiRange::<i32>::full();
    /// assert!(all.is_full() && all.contains(&i32::MAX));
    /// ```
    pub fn full() -> Self {
        MultiRange(vec![AnyRange::full()])
    }

    /// Whether the set holds no values. For integers, check the
    /// [`discrete`](Self::discrete) form.
    ///
    /// ```
    /// use rangecomp::{AnyRange, MultiRange};
    ///
    /// assert!((MultiRange::from(..0) & MultiRange::from(1..)).is_empty());
    ///
    /// let gap = MultiRange::from(AnyRange::greater_than(4)) & MultiRange::from(..5);
    /// assert!(!gap.is_empty());
    /// assert!(gap.discrete().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Whether the set holds every value. For integers, check the
    /// [`discrete`](Self::discrete) form.
    ///
    /// ```
    /// use rangecomp::{AnyRange, MultiRange};
    ///
    /// assert!((MultiRange::from(..5) | MultiRange::from(5..)).is_full());
    ///
    /// let ints = MultiRange::from([AnyRange::at_most(4), AnyRange::at_least(5)]);
    /// assert!(!ints.is_full());
    /// assert!(ints.discrete().is_full());
    /// ```
    pub fn is_full(&self) -> bool {
        matches!(
            self.0.as_slice(),
            [AnyRange {
                lower: NegInf,
                upper: PosInf
            }]
        )
    }

    /// The ranges, in ascending order, as a slice.
    ///
    /// ```
    /// use rangecomp::{AnyRange, MultiRange};
    ///
    /// let set = MultiRange::from([7..9, 1..3]);
    /// assert_eq!(set.as_slice(), [AnyRange::from(1..3), AnyRange::from(7..9)]);
    /// assert_eq!(set.as_slice().len(), 2);
    /// ```
    pub fn as_slice(&self) -> &[AnyRange<T>] {
        &self.0
    }

    /// Iterates over the ranges in ascending order.
    ///
    /// `for range in &set` does the same, and `for range in set` takes the
    /// ranges out of the set.
    ///
    /// ```
    /// use rangecomp::MultiRange;
    ///
    /// let set = MultiRange::from([7..9, 1..3]);
    /// let starts: Vec<_> = set.iter().map(|r| r.lower().value().copied()).collect();
    /// assert_eq!(starts, [Some(1), Some(7)]);
    /// ```
    pub fn iter(&self) -> slice::Iter<'_, AnyRange<T>> {
        self.0.iter()
    }
}

impl<T: Ord> MultiRange<T> {
    /// Whether `v` is in the set.
    ///
    /// This is a binary search, so it takes O(log n) time for n ranges.
    ///
    /// ```
    /// use rangecomp::{AnyRange, MultiRange};
    ///
    /// let set = MultiRange::from([AnyRange::from(1..3), AnyRange::point(5)]);
    /// assert!(set.contains(&1) && set.contains(&5));
    /// assert!(!set.contains(&3) && !set.contains(&4));
    /// ```
    pub fn contains(&self, v: &T) -> bool {
        let idx = self.0.partition_point(|r| r.upper.cmp_value(v).is_lt());
        self.0.get(idx).is_some_and(|r| r.contains(v))
    }

    /// Adds the values in `range`, merging it with any ranges it overlaps or
    /// touches.
    ///
    /// Finding the place takes O(log n) time, but making room shifts the later
    /// ranges. To add many ranges, [`extend`](Extend::extend) or
    /// [`collect`](Iterator::collect) them all at once.
    ///
    /// ```
    /// use rangecomp::{AnyRange, MultiRange};
    ///
    /// let mut s = MultiRange::from([1..3, 7..9]);
    /// s.insert(AnyRange::from(3..7));
    /// assert_eq!(s, MultiRange::from(1..9));
    /// ```
    pub fn insert(&mut self, range: AnyRange<T>) {
        if range.is_empty() {
            return;
        }
        let start = self.0.partition_point(|r| !r.upper.touches(&range.lower));
        // `max` only matters for an `Ord` that is not a total order.
        let end = self
            .0
            .partition_point(|r| range.upper.touches(&r.lower))
            .max(start);
        let merged = self.0.drain(start..end).fold(range, |acc, r| AnyRange {
            lower: acc.lower.min(r.lower),
            upper: acc.upper.max(r.upper),
        });
        self.0.insert(start, merged);
    }

    /// Whether every value in `self` is also in `other`. For integers,
    /// compare the [`discrete`](Self::discrete) forms of both sets.
    ///
    /// ```
    /// use rangecomp::MultiRange;
    ///
    /// let small = MultiRange::from(3..5);
    /// let big = MultiRange::from(0..10);
    /// assert!(small.is_subset(&big));
    /// assert!(!big.is_subset(&small));
    /// assert!(MultiRange::empty().is_subset(&small));
    /// ```
    pub fn is_subset(&self, other: &Self) -> bool {
        self.0.iter().all(|a| {
            let idx = other.0.partition_point(|b| b.upper < a.upper);
            other.0.get(idx).is_some_and(|b| b.lower <= a.lower)
        })
    }

    /// Whether every value in `other` is also in `self`.
    ///
    /// ```
    /// use rangecomp::MultiRange;
    ///
    /// assert!(MultiRange::from(0..10).is_superset(&MultiRange::from(3..5)));
    /// ```
    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    /// Whether the sets share no value.
    ///
    /// ```
    /// use rangecomp::MultiRange;
    ///
    /// assert!(MultiRange::from(0..5).is_disjoint(&MultiRange::from(5..10)));
    /// assert!(!MultiRange::from(0..=5).is_disjoint(&MultiRange::from(5..10)));
    /// ```
    pub fn is_disjoint(&self, other: &Self) -> bool {
        let (mut i, mut j) = (0, 0);
        while let (Some(a), Some(b)) = (self.0.get(i), other.0.get(j)) {
            if a.intersects(b) {
                return false;
            }
            if a.upper < b.upper {
                i += 1;
            } else {
                j += 1;
            }
        }
        true
    }

    /// The values, if the set is made of single values only.
    ///
    /// Returns `None` if any range holds more than one value. The empty set
    /// gives an empty list. For a [`Discrete`] type, [`values`](Self::values)
    /// also lists the values inside wider ranges such as `1..=3`.
    ///
    /// ```
    /// use rangecomp::{AnyRange, MultiRange};
    ///
    /// let in_list = MultiRange::from([3, 1, 2].map(AnyRange::point));
    /// assert_eq!(in_list.points(), Some(vec![&1, &2, &3]));
    /// assert_eq!(MultiRange::from(1..5).points(), None);
    /// ```
    pub fn points(&self) -> Option<Vec<&T>> {
        self.0.iter().map(AnyRange::as_point).collect()
    }
}

impl<T: Ord + Clone> MultiRange<T> {
    /// The values in either set. The same as `&a | &b`.
    ///
    /// Takes O(n + m) time.
    ///
    /// ```
    /// use rangecomp::MultiRange;
    ///
    /// let a = MultiRange::from(0..5);
    /// let b = MultiRange::from(3..8);
    /// assert_eq!(a.union(&b), MultiRange::from(0..8));
    /// assert_eq!(a.union(&MultiRange::from(10..12)), MultiRange::from([0..5, 10..12]));
    /// ```
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        merge(self.0.iter().cloned(), other.0.iter().cloned())
    }

    /// The values in both sets. The same as `&a & &b`.
    ///
    /// Takes O(n + m) time.
    ///
    /// ```
    /// use rangecomp::MultiRange;
    ///
    /// let a = MultiRange::from([0..5, 10..15]);
    /// let b = MultiRange::from(3..12);
    /// assert_eq!(a.intersection(&b), MultiRange::from([3..5, 10..12]));
    /// ```
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Self {
        let mut out = Vec::new();
        let (mut i, mut j) = (0, 0);
        while let (Some(a), Some(b)) = (self.0.get(i), other.0.get(j)) {
            let both = a.intersection(b);
            if !both.is_empty() {
                out.push(both);
            }
            if a.upper < b.upper {
                i += 1;
            } else {
                j += 1;
            }
        }
        MultiRange(out)
    }

    /// The values not in the set. The same as `!&a`.
    ///
    /// Takes O(n) time.
    ///
    /// ```
    /// use rangecomp::{AnyRange, MultiRange};
    ///
    /// assert_eq!(MultiRange::from(..5).complement(), MultiRange::from(5..));
    ///
    /// let not_five = MultiRange::from(AnyRange::point(5)).complement();
    /// assert_eq!(not_five.as_slice(), [AnyRange::less_than(5), AnyRange::greater_than(5)]);
    /// ```
    #[must_use]
    pub fn complement(&self) -> Self {
        complement(self.0.iter().cloned())
    }

    /// The values in `self` but not in `other`. The same as `&a - &b`.
    ///
    /// Takes O(n + m) time.
    ///
    /// ```
    /// use rangecomp::MultiRange;
    ///
    /// let a = MultiRange::from(0..10);
    /// assert_eq!(a.difference(&MultiRange::from(3..5)), MultiRange::from([0..3, 5..10]));
    /// ```
    #[must_use]
    pub fn difference(&self, other: &Self) -> Self {
        let mut out = Vec::new();
        let mut j = 0;
        for a in &self.0 {
            while other.0.get(j).is_some_and(|b| b.upper < a.lower) {
                j += 1;
            }
            // Cut each overlapping range of `other` out of `a`.
            let mut lower = a.lower.clone();
            for b in other.0[j..].iter().take_while(|b| b.lower <= a.upper) {
                out.push(AnyRange::new(lower, b.lower.clone().end_before()));
                lower = b.upper.clone().start_after();
            }
            out.push(AnyRange::new(lower, a.upper.clone()));
        }
        out.retain(|r| !r.is_empty());
        MultiRange(out)
    }

    /// The values in exactly one of the sets. The same as `&a ^ &b`.
    ///
    /// ```
    /// use rangecomp::MultiRange;
    ///
    /// let a = MultiRange::from(0..10);
    /// let b = MultiRange::from(5..15);
    /// assert_eq!(a.symmetric_difference(&b), MultiRange::from([0..5, 10..15]));
    /// ```
    #[must_use]
    pub fn symmetric_difference(&self, other: &Self) -> Self {
        self.difference(other) | other.difference(self)
    }

    /// The smallest single range holding the whole set, or `None` if the set
    /// is empty.
    ///
    /// It runs from the start of the first range to the end of the last, so
    /// it fills in any gaps.
    ///
    /// ```
    /// use rangecomp::{AnyRange, MultiRange};
    ///
    /// let set = MultiRange::from([1..3, 7..9]);
    /// assert_eq!(set.hull(), Some(AnyRange::from(1..9)));
    /// assert_eq!(MultiRange::<i32>::empty().hull(), None);
    /// ```
    pub fn hull(&self) -> Option<AnyRange<T>> {
        let (first, last) = (self.0.first()?, self.0.last()?);
        Some(AnyRange {
            lower: first.lower.clone(),
            upper: last.upper.clone(),
        })
    }
}

impl<T: Discrete> MultiRange<T> {
    /// The same values over a [`Discrete`] type, in a form where sets holding
    /// the same values are equal.
    ///
    /// Open bounds become closed and ranges with no value between them merge.
    /// A lower end at the type's smallest value becomes `NegInf`, and an upper
    /// end at its largest value becomes `PosInf`.
    ///
    /// Call this (on both sets, where there are two) before relying on
    /// equality, [`is_empty`](Self::is_empty), [`is_full`](Self::is_full),
    /// [`is_subset`](Self::is_subset), [`is_superset`](Self::is_superset),
    /// [`is_disjoint`](Self::is_disjoint), [`points`](Self::points) or
    /// [`hull`](Self::hull). [`contains`](Self::contains),
    /// [`values`](Self::values) and the set operations are exact either way.
    ///
    /// ```
    /// use rangecomp::{AnyRange, MultiRange};
    ///
    /// let s = MultiRange::from([AnyRange::at_most(4), AnyRange::at_least(5)]);
    /// assert!(!s.is_full());
    /// assert!(s.discrete().is_full());
    ///
    /// let both = MultiRange::from([AnyRange::point(false), AnyRange::point(true)]);
    /// assert!(both.discrete().is_full());
    /// ```
    #[must_use]
    pub fn discrete(&self) -> Self {
        let mut ranges: Vec<_> = self
            .0
            .iter()
            .map(|r| r.clone().discrete())
            .filter(|r| !r.is_empty())
            .collect();
        // Already sorted unless a `Discrete` impl breaks its rules, and sorting
        // sorted input takes linear time.
        ranges.sort_unstable_by(|a, b| a.lower.cmp(&b.lower));
        let mut out: Vec<AnyRange<T>> = Vec::with_capacity(ranges.len());
        for range in ranges {
            match out.last_mut() {
                Some(last)
                    if last.upper.touches(&range.lower) || follows(&last.upper, &range.lower) =>
                {
                    if range.upper > last.upper {
                        last.upper = range.upper;
                    }
                }
                _ => out.push(range),
            }
        }
        MultiRange(out)
    }

    /// The values in the set, in ascending order, produced lazily.
    ///
    /// Returns `None` only if the set is unbounded and `T` has no
    /// [`MIN`](Discrete::MIN) or [`MAX`](Discrete::MAX) to stop at. The
    /// built-in types have both, so `x > 5` over `i64` yields every value up
    /// to `i64::MAX`; use [`take`](Iterator::take) to cap it.
    ///
    /// ```
    /// use rangecomp::{AnyRange, MultiRange};
    ///
    /// let s = MultiRange::from([AnyRange::from(1..4), AnyRange::point(7)]);
    /// let values: Vec<i32> = s.values().unwrap().collect();
    /// assert_eq!(values, [1, 2, 3, 7]);
    ///
    /// let not_true = !MultiRange::from(AnyRange::point(true));
    /// assert_eq!(not_true.values().unwrap().collect::<Vec<_>>(), [false]);
    /// ```
    pub fn values(&self) -> Option<impl Iterator<Item = T>> {
        let bounds = self
            .discrete()
            .0
            .into_iter()
            .map(|r| Some((closed(r.lower, T::MIN)?, closed(r.upper, T::MAX)?)))
            .collect::<Option<Vec<_>>>()?;
        Some(
            bounds.into_iter().flat_map(|(lo, hi)| {
                iter::successors(Some(lo), move |v| next(v).filter(|n| *n <= hi))
            }),
        )
    }
}

/// Merges two sorted lists of normalized ranges.
fn merge<T: Ord>(
    a: impl IntoIterator<Item = AnyRange<T>>,
    b: impl IntoIterator<Item = AnyRange<T>>,
) -> MultiRange<T> {
    let (mut a, mut b) = (a.into_iter().peekable(), b.into_iter().peekable());
    let mut out = Vec::new();
    loop {
        let next = match (a.peek(), b.peek()) {
            (Some(x), Some(y)) if y.lower < x.lower => b.next(),
            (Some(_), _) => a.next(),
            (None, _) => b.next(),
        };
        match next {
            Some(range) => push_merged(&mut out, range),
            None => return MultiRange(out),
        }
    }
}

/// Appends a range that starts no earlier than the last one, merging the two
/// if they overlap or touch.
fn push_merged<T: Ord>(out: &mut Vec<AnyRange<T>>, next: AnyRange<T>) {
    match out.last_mut() {
        Some(last) if last.upper.touches(&next.lower) => {
            if next.upper > last.upper {
                last.upper = next.upper;
            }
        }
        _ => out.push(next),
    }
}

/// The gaps around a sorted list of normalized ranges.
fn complement<T: Ord>(ranges: impl ExactSizeIterator<Item = AnyRange<T>>) -> MultiRange<T> {
    let mut out = Vec::with_capacity(ranges.len() + 1);
    let mut lower = NegInf;
    for r in ranges {
        out.push(AnyRange::new(lower, r.lower.end_before()));
        lower = r.upper.start_after();
    }
    out.push(AnyRange::new(lower, PosInf));
    out.retain(|r| !r.is_empty());
    MultiRange(out)
}

/// Whether a discrete range starting at `lower` begins at the value right
/// after one ending at `upper`.
fn follows<T: Discrete>(upper: &Boundary<T>, lower: &Boundary<T>) -> bool {
    matches!((upper, lower), (At(u), At(l)) if next(u).as_ref() == Some(l))
}

/// The value at a closed bound, or `edge` for an infinite one.
fn closed<T>(bound: Boundary<T>, edge: Option<T>) -> Option<T> {
    match bound {
        At(v) => Some(v),
        NegInf | PosInf => edge,
        Boundary::Below(_) | Boundary::Above(_) => unreachable!("discrete bounds are closed"),
    }
}

impl<T: fmt::Display + PartialEq> fmt::Display for MultiRange<T> {
    /// Writes the ranges in interval notation, joined by `∪`, or `∅` for the
    /// empty set.
    ///
    /// ```
    /// use rangecomp::{AnyRange, MultiRange};
    ///
    /// let set = MultiRange::from([AnyRange::less_than(0), AnyRange::point(5), AnyRange::from(10..)]);
    /// assert_eq!(set.to_string(), "(-inf, 0) ∪ {5} ∪ [10, inf)");
    /// assert_eq!(MultiRange::<i32>::empty().to_string(), "∅");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str("∅");
        }
        for (i, range) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(" ∪ ")?;
            }
            range.fmt(f)?;
        }
        Ok(())
    }
}

impl<T> Default for MultiRange<T> {
    fn default() -> Self {
        MultiRange::empty()
    }
}

impl<T: Ord> From<AnyRange<T>> for MultiRange<T> {
    fn from(range: AnyRange<T>) -> Self {
        if range.is_empty() {
            MultiRange::empty()
        } else {
            MultiRange(vec![range])
        }
    }
}

macro_rules! from_range {
    ($($ty:ty),*) => {$(
        impl<T: Ord> From<$ty> for MultiRange<T> {
            /// The set holding the same values as the [`AnyRange`] it converts to.
            fn from(range: $ty) -> Self {
                AnyRange::from(range).into()
            }
        }
    )*};
}

from_range!(
    Range<T>,
    RangeInclusive<T>,
    RangeFrom<T>,
    RangeTo<T>,
    RangeToInclusive<T>,
    RangeFull,
    (Bound<T>, Bound<T>),
    (Boundary<T>, Boundary<T>)
);

impl<T: Ord, R: Into<AnyRange<T>>, const N: usize> From<[R; N]> for MultiRange<T> {
    /// The union of the ranges, which can be [`AnyRange`]s or anything that
    /// converts to one, such as `1..5`.
    fn from(ranges: [R; N]) -> Self {
        ranges.into_iter().map(Into::into).collect()
    }
}

impl<T: Ord> FromIterator<AnyRange<T>> for MultiRange<T> {
    /// The union of the ranges, in any order. Takes O(n log n) time.
    fn from_iter<I: IntoIterator<Item = AnyRange<T>>>(iter: I) -> Self {
        let mut ranges: Vec<_> = iter.into_iter().filter(|r| !r.is_empty()).collect();
        ranges.sort_unstable_by(|a, b| a.lower.cmp(&b.lower));
        let mut out = Vec::with_capacity(ranges.len());
        for range in ranges {
            push_merged(&mut out, range);
        }
        MultiRange(out)
    }
}

impl<T: Ord> Extend<AnyRange<T>> for MultiRange<T> {
    /// Adds the values in each range. Takes O((n + k) log (n + k)) time.
    fn extend<I: IntoIterator<Item = AnyRange<T>>>(&mut self, iter: I) {
        let ranges = mem::take(&mut self.0);
        *self = ranges.into_iter().chain(iter).collect();
    }
}

impl<T> IntoIterator for MultiRange<T> {
    type Item = AnyRange<T>;
    type IntoIter = vec::IntoIter<AnyRange<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a MultiRange<T> {
    type Item = &'a AnyRange<T>;
    type IntoIter = slice::Iter<'a, AnyRange<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T: Ord> BitOr for MultiRange<T> {
    type Output = Self;

    /// The union, reusing both sets' ranges.
    fn bitor(self, rhs: Self) -> Self {
        merge(self.0, rhs.0)
    }
}

impl<T: Ord + Clone> BitOr for &MultiRange<T> {
    type Output = MultiRange<T>;

    /// The same as [`MultiRange::union`].
    fn bitor(self, rhs: Self) -> MultiRange<T> {
        self.union(rhs)
    }
}

impl<T: Ord + Clone> BitAnd for MultiRange<T> {
    type Output = Self;

    /// The same as [`MultiRange::intersection`].
    fn bitand(self, rhs: Self) -> Self {
        self.intersection(&rhs)
    }
}

impl<T: Ord + Clone> BitAnd for &MultiRange<T> {
    type Output = MultiRange<T>;

    /// The same as [`MultiRange::intersection`].
    fn bitand(self, rhs: Self) -> MultiRange<T> {
        self.intersection(rhs)
    }
}

impl<T: Ord + Clone> Sub for MultiRange<T> {
    type Output = Self;

    /// The same as [`MultiRange::difference`].
    fn sub(self, rhs: Self) -> Self {
        self.difference(&rhs)
    }
}

impl<T: Ord + Clone> Sub for &MultiRange<T> {
    type Output = MultiRange<T>;

    /// The same as [`MultiRange::difference`].
    fn sub(self, rhs: Self) -> MultiRange<T> {
        self.difference(rhs)
    }
}

impl<T: Ord + Clone> BitXor for MultiRange<T> {
    type Output = Self;

    /// The same as [`MultiRange::symmetric_difference`].
    fn bitxor(self, rhs: Self) -> Self {
        self.symmetric_difference(&rhs)
    }
}

impl<T: Ord + Clone> BitXor for &MultiRange<T> {
    type Output = MultiRange<T>;

    /// The same as [`MultiRange::symmetric_difference`].
    fn bitxor(self, rhs: Self) -> MultiRange<T> {
        self.symmetric_difference(rhs)
    }
}

impl<T: Ord> Not for MultiRange<T> {
    type Output = Self;

    /// The complement, reusing the set's bounds.
    fn not(self) -> Self {
        complement(self.0.into_iter())
    }
}

impl<T: Ord + Clone> Not for &MultiRange<T> {
    type Output = MultiRange<T>;

    /// The same as [`MultiRange::complement`].
    fn not(self) -> MultiRange<T> {
        self.complement()
    }
}

#[cfg(test)]
mod tests {
    use super::MultiRange;
    use crate::{
        AnyRange,
        Boundary::{Above, At, Below, NegInf, PosInf},
    };

    #[test]
    fn exact_adjacency() {
        // `< 5` and `>= 5` merge.
        assert!(MultiRange::from([AnyRange::less_than(5), AnyRange::at_least(5)]).is_full());
        // `<= 5` and `> 5` merge.
        assert!(MultiRange::from([AnyRange::at_most(5), AnyRange::greater_than(5)]).is_full());
        // `< 5` and `> 5` leave 5 out.
        let not_five = MultiRange::from([AnyRange::less_than(5), AnyRange::greater_than(5)]);
        assert_eq!(not_five.as_slice().len(), 2);
        assert!(!not_five.contains(&5));
        // Two half-open ranges sharing an end.
        assert_eq!(MultiRange::from([1..5, 5..9]), MultiRange::from(1..9));
    }

    #[test]
    fn normalizes_unsorted_and_overlapping_input() {
        let s = MultiRange::from([3..7, 10..15, 1..5, 1..2, 5..5, 13..40]);
        assert_eq!(s.as_slice(), [(1..7).into(), (10..40).into()]);
    }

    #[test]
    fn intersection_of_disjoint_sets_is_empty() {
        let a = MultiRange::from([1..3, 10..12]);
        let b = MultiRange::from([4..6, 20..22]);
        assert!(a.intersection(&b).is_empty());
        assert!(a.is_disjoint(&b));
        assert!(MultiRange::from(1..5)
            .intersection(&MultiRange::from(5..9))
            .is_empty());
    }

    #[test]
    fn intersection_merges_sorted_lists() {
        let a = MultiRange::from([0..10, 20..30]);
        let b = MultiRange::from([5..25, 28..40]);
        assert_eq!(
            a.intersection(&b),
            MultiRange::from([5..10, 20..25, 28..30])
        );
        let at_5 = MultiRange::from(AnyRange::point(5));
        assert_eq!(MultiRange::from(..=5).intersection(&at_5), at_5);
        assert!(MultiRange::from(..5).intersection(&at_5).is_empty());
    }

    #[test]
    fn union_merges_sorted_lists() {
        let a = MultiRange::from([0..2, 10..12]);
        let b = MultiRange::from([2..4, 20..22]);
        assert_eq!(a.union(&b), MultiRange::from([0..4, 10..12, 20..22]));
        assert_eq!(a.clone() | b.clone(), a.union(&b));
    }

    #[test]
    fn complement() {
        let not_five = MultiRange::from(AnyRange::point(5)).complement();
        assert_eq!(
            not_five.as_slice(),
            [AnyRange::less_than(5), AnyRange::greater_than(5)]
        );
        assert_eq!(not_five.complement(), MultiRange::from(AnyRange::point(5)));
        assert_eq!(MultiRange::from(..5).complement(), MultiRange::from(5..));
        assert!(MultiRange::<i32>::full().complement().is_empty());
        assert!(MultiRange::<i32>::empty().complement().is_full());
        assert_eq!(!not_five.clone(), not_five.complement());
    }

    #[test]
    fn difference_and_subset() {
        let a = MultiRange::from(0..10);
        let b = MultiRange::from(3..5);
        assert_eq!(a.difference(&b), MultiRange::from([0..3, 5..10]));
        assert!(b.is_subset(&a) && !a.is_subset(&b));
        assert!(a.is_superset(&b));
        assert!(MultiRange::empty().is_subset(&b));
        // A range that spans a gap is not a subset.
        assert!(!MultiRange::from(0..10).is_subset(&MultiRange::from([0..5, 6..10])));
    }

    #[test]
    fn insert() {
        let mut s = MultiRange::empty();
        for r in [5..6, 1..2, 9..10, 2..5, 6..7, 20..20] {
            s.insert(r.into());
        }
        assert_eq!(s, MultiRange::from([1..7, 9..10]));
        s.insert(AnyRange::full());
        assert!(s.is_full());
    }

    #[test]
    fn extend() {
        let mut s = MultiRange::from(1..3);
        s.extend([AnyRange::from(5..7), AnyRange::from(3..5)]);
        assert_eq!(s, MultiRange::from(1..7));
    }

    #[test]
    fn contains() {
        let s = MultiRange::from([
            AnyRange::from(1..3),
            AnyRange::point(5),
            AnyRange::greater_than(8),
        ]);
        let members: Vec<i32> = (0..11).filter(|v| s.contains(v)).collect();
        assert_eq!(members, [1, 2, 5, 9, 10]);
        assert!(!MultiRange::empty().contains(&0));
    }

    #[test]
    fn hull_and_points() {
        let s = MultiRange::from([AnyRange::point(1), AnyRange::point(5)]);
        assert_eq!(s.hull(), Some(AnyRange::between(1, 5)));
        assert_eq!(s.points(), Some(vec![&1, &5]));
        assert_eq!(MultiRange::from(1..5).points(), None);
        assert_eq!(MultiRange::<i32>::empty().points(), Some(vec![]));
        assert_eq!(MultiRange::<i32>::empty().hull(), None);
        assert_eq!(
            MultiRange::from([AnyRange::less_than(1), AnyRange::point(5)]).hull(),
            Some(AnyRange::at_most(5))
        );
    }

    #[test]
    fn canonical_bounds_compare_equal() {
        assert_eq!(
            MultiRange::from((Below(1), Above(5))),
            MultiRange::from(1..=5)
        );
        assert_eq!(
            MultiRange::from([(NegInf, At(5)), (At(5), PosInf)]),
            MultiRange::full()
        );
    }

    #[test]
    fn strings() {
        let s = MultiRange::from([
            AnyRange::point("b".to_string()),
            AnyRange::at_least("x".into()),
        ]);
        assert!(s.contains(&"b".to_string()));
        assert!(!s.contains(&"c".to_string()));
        assert!(s.contains(&"xyz".to_string()));
    }
}
