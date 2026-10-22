use std::{
    fmt,
    ops::{Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use crate::{
    discrete::{next, prev},
    Boundary::{self, Above, At, Below, NegInf, PosInf},
    Discrete,
};

/// A contiguous range of values between a lower and an upper [`Boundary`].
///
/// A range holds every `v` with `lower <= At(v) <= upper`. Build one with a
/// constructor such as [`less_than`](Self::less_than) or
/// [`between`](Self::between), or convert from Rust range syntax, a pair of
/// [`Bound`]s, or a pair of [`Boundary`]s. The [crate
/// docs](crate#building-ranges-and-sets) list every conversion.
///
/// ```
/// use std::ops::Bound::{Excluded, Included};
/// use rangecomp::{AnyRange, Boundary::{Above, At}};
///
/// let r = AnyRange::from((Excluded(1), Included(5)));
/// assert_eq!(r, AnyRange::new(Above(1), At(5)));
/// assert!(!r.contains(&1) && r.contains(&5));
/// assert_eq!(r.to_string(), "(1, 5]");
/// ```
///
/// Ranges are kept in a normal form, so equal ranges compare `==`: the lower
/// end is always `NegInf`, `At` or `Above`, the upper end is always `Below`,
/// `At` or `PosInf`, and every empty range is [`empty()`](Self::empty).
///
/// # Interval relations
///
/// [`before`](Self::before), [`meets`](Self::meets),
/// [`overlaps`](Self::overlaps), [`starts`](Self::starts),
/// [`during`](Self::during), [`finishes`](Self::finishes),
/// [`equals`](Self::equals) and their inverses are the thirteen relations of
/// [Allen's interval algebra]. They compare ends as boundaries, so `[1, 5)`
/// meets `[5, 9]` while `[1, 5)` is before `(5, 9]`.
///
/// For two non-empty ranges that are not single values exactly one relation
/// holds; a single value can satisfy more than one, as the point 5 both meets
/// and starts `[5, 9]`. All thirteen are false if either range is empty.
///
/// [Allen's interval algebra]: https://en.wikipedia.org/wiki/Allen%27s_interval_algebra
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnyRange<T> {
    pub(crate) lower: Boundary<T>,
    pub(crate) upper: Boundary<T>,
}

impl<T> AnyRange<T> {
    /// The range with no values. Every empty range is equal to it.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert_eq!(AnyRange::from(5..1), AnyRange::empty());
    /// ```
    pub fn empty() -> Self {
        AnyRange {
            lower: PosInf,
            upper: NegInf,
        }
    }

    /// Every value, like `..`.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::full().contains(&i64::MIN));
    /// ```
    pub fn full() -> Self {
        AnyRange {
            lower: NegInf,
            upper: PosInf,
        }
    }

    /// Values less than `v`, like `..v`.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(!AnyRange::less_than(5).contains(&5));
    /// ```
    pub fn less_than(v: T) -> Self {
        AnyRange {
            lower: NegInf,
            upper: Below(v),
        }
    }

    /// Values up to and including `v`, like `..=v`.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::at_most(5).contains(&5));
    /// ```
    pub fn at_most(v: T) -> Self {
        AnyRange {
            lower: NegInf,
            upper: At(v),
        }
    }

    /// Values greater than `v`. Range syntax can't express this, but
    /// `(Excluded(v), Unbounded)` converts to the same range.
    ///
    /// ```
    /// use std::ops::Bound::{Excluded, Unbounded};
    /// use rangecomp::AnyRange;
    ///
    /// assert_eq!(AnyRange::greater_than(5), AnyRange::from((Excluded(5), Unbounded)));
    /// ```
    pub fn greater_than(v: T) -> Self {
        AnyRange {
            lower: Above(v),
            upper: PosInf,
        }
    }

    /// Values from `v` up, like `v..`.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::at_least(5).contains(&5));
    /// ```
    pub fn at_least(v: T) -> Self {
        AnyRange {
            lower: At(v),
            upper: PosInf,
        }
    }

    /// The lower end.
    ///
    /// This is `NegInf` if there is no lower limit, `At(v)` for `x >= v`, or
    /// `Above(v)` for `x > v`. For the empty range it is `PosInf`.
    ///
    /// ```
    /// use rangecomp::{AnyRange, Boundary};
    ///
    /// assert_eq!(AnyRange::greater_than(5).lower(), &Boundary::Above(5));
    /// assert_eq!(AnyRange::from(1..5).lower(), &Boundary::At(1));
    /// assert_eq!(AnyRange::less_than(5).lower(), &Boundary::NegInf);
    /// ```
    pub fn lower(&self) -> &Boundary<T> {
        &self.lower
    }

    /// The upper end.
    ///
    /// This is `PosInf` if there is no upper limit, `At(v)` for `x <= v`, or
    /// `Below(v)` for `x < v`. For the empty range it is `NegInf`.
    ///
    /// ```
    /// use rangecomp::{AnyRange, Boundary};
    ///
    /// assert_eq!(AnyRange::from(1..5).upper(), &Boundary::Below(5));
    /// assert_eq!(AnyRange::at_most(5).upper(), &Boundary::At(5));
    /// assert_eq!(AnyRange::at_least(5).upper(), &Boundary::PosInf);
    /// ```
    pub fn upper(&self) -> &Boundary<T> {
        &self.upper
    }

    /// Takes the range apart into its lower and upper ends.
    ///
    /// ```
    /// use rangecomp::{AnyRange, Boundary::{At, Below}};
    ///
    /// assert_eq!(AnyRange::from(1..5).into_bounds(), (At(1), Below(5)));
    /// ```
    pub fn into_bounds(self) -> (Boundary<T>, Boundary<T>) {
        (self.lower, self.upper)
    }

    /// The ends as std [`Bound`]s, or `None` if the range is empty.
    ///
    /// `NegInf` and `PosInf` become `Unbounded`, `At` becomes `Included`, and
    /// `Above` and `Below` become `Excluded`. Ranges inside a
    /// [`MultiRange`](crate::MultiRange) are never empty, so for them this is
    /// always `Some`.
    ///
    /// `AnyRange::from` turns the pair back into an `AnyRange<&T>`; call
    /// [`Bound::cloned`] on each end first for an owned range. The pair also
    /// implements
    /// [`RangeBounds`](std::ops::RangeBounds), so it works with APIs such as
    /// [`BTreeMap::range`](std::collections::BTreeMap::range).
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use std::ops::Bound::{Excluded, Unbounded};
    /// use rangecomp::AnyRange;
    ///
    /// let gt2 = AnyRange::greater_than(2);
    /// assert_eq!(gt2.to_bounds(), Some((Excluded(&2), Unbounded)));
    /// assert_eq!(AnyRange::<i32>::empty().to_bounds(), None);
    ///
    /// let map = BTreeMap::from([(1, "a"), (3, "b"), (5, "c")]);
    /// let found: Vec<_> = map.range(gt2.to_bounds().unwrap()).map(|(_, v)| *v).collect();
    /// assert_eq!(found, ["b", "c"]);
    /// ```
    pub fn to_bounds(&self) -> Option<(Bound<&T>, Bound<&T>)> {
        let lower = match &self.lower {
            NegInf => Bound::Unbounded,
            Below(v) | At(v) => Bound::Included(v),
            Above(v) => Bound::Excluded(v),
            PosInf => return None,
        };
        let upper = match &self.upper {
            PosInf => Bound::Unbounded,
            At(v) | Above(v) => Bound::Included(v),
            Below(v) => Bound::Excluded(v),
            NegInf => return None,
        };
        Some((lower, upper))
    }

    /// Borrows the values at both ends, giving an `AnyRange<&T>`.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// let r = AnyRange::from("a".to_string().."c".to_string());
    /// assert!(r.as_ref().contains(&&"b".to_string()));
    /// ```
    pub fn as_ref(&self) -> AnyRange<&T> {
        AnyRange {
            lower: self.lower.as_ref(),
            upper: self.upper.as_ref(),
        }
    }
}

impl<T: Clone> AnyRange<T> {
    /// Just the value `v`, like `v..=v`.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert_eq!(AnyRange::point(5).as_point(), Some(&5));
    /// ```
    pub fn point(v: T) -> Self {
        AnyRange {
            lower: At(v.clone()),
            upper: At(v),
        }
    }
}

impl<T: Ord> AnyRange<T> {
    /// The range from `lower` to `upper`, in normal form.
    ///
    /// As a lower bound `Below(v)` admits the same values as `At(v)`, and as an
    /// upper bound `Above(v)` admits the same values as `At(v)`, so both are
    /// stored as `At(v)`. A range with no values becomes
    /// [`empty()`](Self::empty).
    ///
    /// ```
    /// use rangecomp::{AnyRange, Boundary::{Above, At, Below}};
    ///
    /// assert_eq!(AnyRange::new(Below(1), Above(5)), AnyRange::between(1, 5));
    /// assert_eq!(AnyRange::new(At(5), Below(5)), AnyRange::empty());
    /// ```
    pub fn new(lower: Boundary<T>, upper: Boundary<T>) -> Self {
        let lower = match lower {
            Below(v) => At(v),
            b => b,
        };
        let upper = match upper {
            Above(v) => At(v),
            b => b,
        };
        if matches!(lower, PosInf) || matches!(upper, NegInf) || lower > upper {
            AnyRange::empty()
        } else {
            AnyRange { lower, upper }
        }
    }

    /// Values from `lo` to `hi` inclusive, like `lo..=hi`. Empty if `lo > hi`.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(1, 5).contains(&5));
    /// ```
    pub fn between(lo: T, hi: T) -> Self {
        AnyRange::new(At(lo), At(hi))
    }

    /// Whether the range holds no values. For integers, check the
    /// [`discrete`](Self::discrete) form: `4 < x < 5` is not empty until then.
    ///
    /// ```
    /// use rangecomp::{AnyRange, Boundary::{Above, Below}};
    ///
    /// assert!(AnyRange::from(5..5).is_empty());
    /// assert!(!AnyRange::point(5).is_empty());
    /// let gap = AnyRange::new(Above(4), Below(5));
    /// assert!(!gap.is_empty());
    /// assert!(gap.discrete().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.lower > self.upper
    }

    /// Whether `v` lies in the range.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// let r = AnyRange::from(1..5);
    /// assert!(r.contains(&1) && r.contains(&4));
    /// assert!(!r.contains(&5));
    /// ```
    pub fn contains(&self, v: &T) -> bool {
        self.lower.cmp_value(v).is_le() && self.upper.cmp_value(v).is_ge()
    }

    /// The value, if the range holds exactly one: `[At(v), At(v)]`.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert_eq!(AnyRange::point(5).as_point(), Some(&5));
    /// assert_eq!(AnyRange::between(5, 6).as_point(), None);
    /// ```
    pub fn as_point(&self) -> Option<&T> {
        match (&self.lower, &self.upper) {
            (At(a), At(b)) if a == b => Some(a),
            _ => None,
        }
    }

    fn both_nonempty(&self, other: &Self) -> bool {
        !self.is_empty() && !other.is_empty()
    }

    /// Whether `self` ends before `other` starts, with values between them.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(1, 3).before(&AnyRange::between(5, 9)));
    /// // 5 lies between `[1, 5)` and `(5, 9]`.
    /// assert!(AnyRange::from(1..5).before(&AnyRange::greater_than(5)));
    /// assert!(!AnyRange::between(1, 5).before(&AnyRange::between(5, 9)));
    /// ```
    pub fn before(&self, other: &Self) -> bool {
        self.both_nonempty(other) && !self.upper.touches(&other.lower)
    }

    /// Whether `self` ends where `other` starts, with no value between them.
    ///
    /// Either both include the meeting value, as `[1, 5]` and `[5, 9]`, or
    /// exactly one does, as `[1, 5)` and `[5, 9]`.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(1, 5).meets(&AnyRange::between(5, 9)));
    /// assert!(AnyRange::from(1..5).meets(&AnyRange::between(5, 9)));
    /// assert!(AnyRange::less_than(5).meets(&AnyRange::at_least(5)));
    /// ```
    pub fn meets(&self, other: &Self) -> bool {
        self.both_nonempty(other) && self.upper <= other.lower && self.upper.touches(&other.lower)
    }

    /// Whether `self` starts first and ends inside `other`.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(1, 5).overlaps(&AnyRange::between(3, 9)));
    /// assert!(!AnyRange::between(3, 9).overlaps(&AnyRange::between(1, 5)));
    /// ```
    pub fn overlaps(&self, other: &Self) -> bool {
        self.both_nonempty(other)
            && self.lower < other.lower
            && other.lower < self.upper
            && self.upper < other.upper
    }

    /// Whether both start at the same boundary and `self` ends first.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(1, 3).starts(&AnyRange::between(1, 9)));
    /// // `[1, 5)` ends just before `[1, 5]` does.
    /// assert!(AnyRange::from(1..5).starts(&AnyRange::between(1, 5)));
    /// ```
    pub fn starts(&self, other: &Self) -> bool {
        self.both_nonempty(other) && self.lower == other.lower && self.upper < other.upper
    }

    /// Whether `self` lies inside `other`, which starts earlier and ends
    /// later.
    ///
    /// Ends compare as boundaries, so `(1, 5)` is during `[1, 5]`.
    ///
    /// ```
    /// use rangecomp::{AnyRange, Boundary::{Above, Below}};
    ///
    /// assert!(AnyRange::between(3, 5).during(&AnyRange::between(1, 9)));
    /// assert!(AnyRange::new(Above(1), Below(5)).during(&AnyRange::between(1, 5)));
    /// assert!(!AnyRange::between(1, 5).during(&AnyRange::between(1, 9)));
    /// ```
    pub fn during(&self, other: &Self) -> bool {
        self.both_nonempty(other) && other.lower < self.lower && self.upper < other.upper
    }

    /// Whether both end at the same boundary and `self` starts later.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(5, 9).finishes(&AnyRange::between(1, 9)));
    /// assert!(AnyRange::greater_than(5).finishes(&AnyRange::at_least(5)));
    /// ```
    pub fn finishes(&self, other: &Self) -> bool {
        self.both_nonempty(other) && other.lower < self.lower && self.upper == other.upper
    }

    /// Whether both have the same ends.
    ///
    /// Unlike `==`, this is false when the ranges are empty, like all the
    /// relations.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(1, 5).equals(&AnyRange::from(1..=5)));
    /// assert!(!AnyRange::<i32>::empty().equals(&AnyRange::empty()));
    /// ```
    pub fn equals(&self, other: &Self) -> bool {
        self.both_nonempty(other) && self == other
    }

    /// Whether `self` starts after `other` ends, with values between them. The
    /// inverse of [`before`](Self::before).
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(5, 9).after(&AnyRange::between(1, 3)));
    /// ```
    pub fn after(&self, other: &Self) -> bool {
        other.before(self)
    }

    /// Whether `self` starts where `other` ends. The inverse of
    /// [`meets`](Self::meets).
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(5, 9).met_by(&AnyRange::between(1, 5)));
    /// ```
    pub fn met_by(&self, other: &Self) -> bool {
        other.meets(self)
    }

    /// Whether `other` starts first and ends inside `self`. The inverse of
    /// [`overlaps`](Self::overlaps).
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(3, 9).overlapped_by(&AnyRange::between(1, 5)));
    /// ```
    pub fn overlapped_by(&self, other: &Self) -> bool {
        other.overlaps(self)
    }

    /// Whether both start together and `self` ends last. The inverse of
    /// [`starts`](Self::starts).
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(1, 9).started_by(&AnyRange::between(1, 3)));
    /// ```
    pub fn started_by(&self, other: &Self) -> bool {
        other.starts(self)
    }

    /// Whether `other` lies inside `self`, away from both ends. The inverse of
    /// [`during`](Self::during).
    ///
    /// Allen calls this relation *contains*; it has a different name here
    /// because [`contains`](Self::contains) tests a single value.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(1, 9).encloses(&AnyRange::between(3, 5)));
    /// ```
    pub fn encloses(&self, other: &Self) -> bool {
        other.during(self)
    }

    /// Whether both end together and `self` starts first. The inverse of
    /// [`finishes`](Self::finishes).
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(1, 9).finished_by(&AnyRange::between(5, 9)));
    /// ```
    pub fn finished_by(&self, other: &Self) -> bool {
        other.finishes(self)
    }

    /// Whether the ranges share at least one value.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::between(1, 5).intersects(&AnyRange::between(5, 9)));
    /// assert!(!AnyRange::from(1..5).intersects(&AnyRange::from(5..9)));
    /// ```
    pub fn intersects(&self, other: &Self) -> bool {
        self.both_nonempty(other) && self.lower <= other.upper && other.lower <= self.upper
    }

    /// Whether the ranges share no value. True if either is empty.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert!(AnyRange::from(1..5).disjoint(&AnyRange::from(5..9)));
    /// assert!(AnyRange::empty().disjoint(&AnyRange::<i32>::full()));
    /// ```
    pub fn disjoint(&self, other: &Self) -> bool {
        !self.intersects(other)
    }
}

impl<T: Ord + Clone> AnyRange<T> {
    /// The values in both ranges, which may be [`empty`](Self::empty).
    ///
    /// The union of two ranges may need two pieces, so it lives on
    /// [`MultiRange`](crate::MultiRange) instead.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// let both = AnyRange::from(1..5).intersection(&AnyRange::between(3, 9));
    /// assert_eq!(both, AnyRange::from(3..5));
    /// assert!(AnyRange::from(1..3).intersection(&AnyRange::from(5..9)).is_empty());
    /// ```
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Self {
        let lower = (&self.lower).max(&other.lower);
        let upper = (&self.upper).min(&other.upper);
        if lower > upper {
            AnyRange::empty()
        } else {
            AnyRange {
                lower: lower.clone(),
                upper: upper.clone(),
            }
        }
    }
}

impl<T: Discrete> AnyRange<T> {
    /// The same values over a [`Discrete`] type, with closed bounds.
    ///
    /// `x > 4` becomes `x >= 5` and `(4, 5)` becomes empty. A lower end at the
    /// type's smallest value becomes `NegInf`, and an upper end at its largest
    /// value becomes `PosInf`, so `x >= 0` over `u8` is
    /// [`full()`](Self::full). Relations then compare the closed
    /// bounds: `[1, 5)` meets `[5, 9]`, but its discrete form `[1, 4]` is
    /// before it.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert_eq!(AnyRange::greater_than(4).discrete(), AnyRange::at_least(5));
    /// assert!(AnyRange::from(5..5).discrete().is_empty());
    /// assert_eq!(AnyRange::at_least(0u8).discrete(), AnyRange::full());
    /// ```
    #[must_use]
    pub fn discrete(self) -> Self {
        let lower = match self.lower {
            At(v) if prev(&v).is_none() => NegInf,
            Above(v) => next(&v).map_or(PosInf, At),
            b => b,
        };
        let upper = match self.upper {
            At(v) if next(&v).is_none() => PosInf,
            Below(v) => prev(&v).map_or(NegInf, At),
            b => b,
        };
        AnyRange::new(lower, upper)
    }
}

impl<T: fmt::Display + PartialEq> fmt::Display for AnyRange<T> {
    /// Writes the range in interval notation: `[1, 5)`, `(-inf, 5]`, `{5}`
    /// for a single value, or `∅` when empty.
    ///
    /// Formatting options such as width or precision apply to each value.
    ///
    /// ```
    /// use rangecomp::AnyRange;
    ///
    /// assert_eq!(AnyRange::from(1..5).to_string(), "[1, 5)");
    /// assert_eq!(AnyRange::greater_than(5).to_string(), "(5, inf)");
    /// assert_eq!(AnyRange::at_most(5).to_string(), "(-inf, 5]");
    /// assert_eq!(AnyRange::point(5).to_string(), "{5}");
    /// assert_eq!(AnyRange::<i32>::full().to_string(), "(-inf, inf)");
    /// assert_eq!(AnyRange::<i32>::empty().to_string(), "∅");
    /// assert_eq!(format!("{:02}", AnyRange::from(1..5)), "[01, 05)");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.lower, &self.upper) {
            (PosInf, _) | (_, NegInf) => f.write_str("∅"),
            (At(a), At(b)) if a == b => {
                f.write_str("{")?;
                a.fmt(f)?;
                f.write_str("}")
            }
            (lower, upper) => {
                match lower {
                    Below(v) | At(v) => {
                        f.write_str("[")?;
                        v.fmt(f)?;
                    }
                    Above(v) => {
                        f.write_str("(")?;
                        v.fmt(f)?;
                    }
                    NegInf | PosInf => f.write_str("(-inf")?,
                }
                f.write_str(", ")?;
                match upper {
                    At(v) | Above(v) => {
                        v.fmt(f)?;
                        f.write_str("]")
                    }
                    Below(v) => {
                        v.fmt(f)?;
                        f.write_str(")")
                    }
                    NegInf | PosInf => f.write_str("inf)"),
                }
            }
        }
    }
}

impl<T: Ord> From<Range<T>> for AnyRange<T> {
    /// `lo..hi` is `[At(lo), Below(hi)]`.
    fn from(r: Range<T>) -> Self {
        AnyRange::new(At(r.start), Below(r.end))
    }
}

impl<T: Ord> From<RangeInclusive<T>> for AnyRange<T> {
    /// `lo..=hi` is `[At(lo), At(hi)]`.
    fn from(r: RangeInclusive<T>) -> Self {
        let (lo, hi) = r.into_inner();
        AnyRange::between(lo, hi)
    }
}

impl<T> From<RangeFrom<T>> for AnyRange<T> {
    /// `lo..` is `[At(lo), PosInf]`.
    fn from(r: RangeFrom<T>) -> Self {
        AnyRange::at_least(r.start)
    }
}

impl<T> From<RangeTo<T>> for AnyRange<T> {
    /// `..hi` is `[NegInf, Below(hi)]`.
    fn from(r: RangeTo<T>) -> Self {
        AnyRange::less_than(r.end)
    }
}

impl<T> From<RangeToInclusive<T>> for AnyRange<T> {
    /// `..=hi` is `[NegInf, At(hi)]`.
    fn from(r: RangeToInclusive<T>) -> Self {
        AnyRange::at_most(r.end)
    }
}

impl<T> From<RangeFull> for AnyRange<T> {
    /// `..` is `[NegInf, PosInf]`.
    fn from(_: RangeFull) -> Self {
        AnyRange::full()
    }
}

impl<T: Ord> From<(Bound<T>, Bound<T>)> for AnyRange<T> {
    /// `Included` is `At` at either end. `Excluded` is `Above` at the lower end
    /// and `Below` at the upper end.
    fn from((lo, hi): (Bound<T>, Bound<T>)) -> Self {
        let lower = match lo {
            Bound::Included(v) => At(v),
            Bound::Excluded(v) => Above(v),
            Bound::Unbounded => NegInf,
        };
        let upper = match hi {
            Bound::Included(v) => At(v),
            Bound::Excluded(v) => Below(v),
            Bound::Unbounded => PosInf,
        };
        AnyRange::new(lower, upper)
    }
}

impl<T: Ord> From<(Boundary<T>, Boundary<T>)> for AnyRange<T> {
    /// The same as [`AnyRange::new`].
    fn from((lower, upper): (Boundary<T>, Boundary<T>)) -> Self {
        AnyRange::new(lower, upper)
    }
}

#[cfg(test)]
mod tests {
    use super::AnyRange;
    use crate::Boundary::{Above, At, Below, NegInf, PosInf};
    use std::ops::Bound;

    #[test]
    fn constructors() {
        assert_eq!(AnyRange::less_than(5).into_bounds(), (NegInf, Below(5)));
        assert_eq!(AnyRange::at_most(5).into_bounds(), (NegInf, At(5)));
        assert_eq!(AnyRange::greater_than(5).into_bounds(), (Above(5), PosInf));
        assert_eq!(AnyRange::at_least(5).into_bounds(), (At(5), PosInf));
        assert_eq!(AnyRange::between(1, 5).into_bounds(), (At(1), At(5)));
        assert_eq!(AnyRange::point(5).into_bounds(), (At(5), At(5)));
        assert_eq!(AnyRange::<i32>::full().into_bounds(), (NegInf, PosInf));
        assert_eq!(AnyRange::<i32>::empty().into_bounds(), (PosInf, NegInf));
    }

    #[test]
    fn new_canonicalizes() {
        assert_eq!(AnyRange::new(Below(1), Above(5)), AnyRange::between(1, 5));
        assert_eq!(
            AnyRange::new(Above(1), Below(5)).into_bounds(),
            (Above(1), Below(5))
        );
    }

    #[test]
    fn emptiness() {
        assert!(!AnyRange::point(5).is_empty());
        assert!(AnyRange::from(5..5).is_empty());
        assert!(AnyRange::new(Above(5), At(5)).is_empty());
        assert!(AnyRange::new(Below(5), Below(5)).is_empty());
        assert!(AnyRange::new(Above(5), Above(5)).is_empty());
        assert!(AnyRange::<i32>::new(PosInf, PosInf).is_empty());
        assert!(AnyRange::<i32>::new(NegInf, NegInf).is_empty());
        // The domain is continuous: (4, 5) is not empty even for integers.
        assert!(!AnyRange::new(Above(4), Below(5)).is_empty());
    }

    #[test]
    fn empties_are_equal() {
        let empty = AnyRange::<i32>::empty();
        assert_eq!(AnyRange::between(5, 4), empty);
        assert_eq!(AnyRange::new(Above(5), Below(5)), empty);
        assert_eq!(
            AnyRange::from((Bound::Excluded(5), Bound::Excluded(5))),
            empty
        );
        assert_eq!(AnyRange::new(PosInf, PosInf), empty);
        assert_eq!(
            AnyRange::from(1..5).intersection(&AnyRange::from(7..9)),
            empty
        );
    }

    #[test]
    fn membership() {
        let r = AnyRange::from(1..5);
        assert!(!r.contains(&0) && r.contains(&1) && r.contains(&4) && !r.contains(&5));
        assert!(AnyRange::full().contains(&i32::MIN));
        assert!(!AnyRange::from(5..5).contains(&5));
        let words = AnyRange::from("b".to_string().."d".to_string());
        assert!(words.contains(&"c".to_string()) && !words.contains(&"d".to_string()));
    }

    #[test]
    fn points() {
        assert_eq!(AnyRange::point(5).as_point(), Some(&5));
        assert_eq!(AnyRange::between(5, 6).as_point(), None);
        assert_eq!(AnyRange::from(5..5).as_point(), None);
    }

    #[test]
    fn intersection() {
        let i = AnyRange::from(1..5).intersection(&AnyRange::between(3, 9));
        assert_eq!(i, AnyRange::from(3..5));
        assert!(AnyRange::from(1..5)
            .intersection(&AnyRange::from(5..9))
            .is_empty());
        assert!(AnyRange::empty()
            .intersection(&AnyRange::<i32>::full())
            .is_empty());
    }

    #[test]
    fn discrete() {
        assert_eq!(
            AnyRange::new(Above(1), Below(5)).discrete(),
            AnyRange::between(2, 4)
        );
        assert!(AnyRange::new(Above(4), Below(5)).discrete().is_empty());
        assert!(AnyRange::greater_than(u8::MAX).discrete().is_empty());
        assert_eq!(
            AnyRange::between(i8::MIN, 0).discrete(),
            AnyRange::at_most(0)
        );
        assert_eq!(AnyRange::point(true).discrete(), AnyRange::at_least(true));
    }

    #[test]
    fn as_ref() {
        let r = AnyRange::from("a".to_string().."c".to_string());
        assert_eq!(
            r.as_ref(),
            AnyRange::from(&"a".to_string()..&"c".to_string())
        );
        assert_eq!(AnyRange::<i32>::empty().as_ref(), AnyRange::empty());
    }
}
