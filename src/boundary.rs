use std::cmp::Ordering;

use Boundary::{Above, At, Below, NegInf, PosInf};

/// One end of a range.
///
/// Each value `v` has three boundaries: `Below(v)` just before it, `At(v)` on
/// it, and `Above(v)` just after it. With the two infinities they form one
/// ordered line, with nothing between `Below(v)` and `At(v)` or between
/// `At(v)` and `Above(v)`:
///
/// ```text
/// NegInf < Below(4) < At(4) < Above(4) < Below(5) < At(5) < Above(5) < PosInf
/// ```
///
/// This is how a range tells open ends from closed ones: `x < 5` ends at
/// `Below(5)`, `x <= 5` at `At(5)`, and `x > 5` starts at `Above(5)`.
///
/// ```
/// use rangecomp::Boundary::{Above, At, Below, NegInf, PosInf};
///
/// assert!(NegInf < Below(5) && Below(5) < At(5) && At(5) < Above(5));
/// assert!(Above(4) < Below(5) && Above(5) < PosInf);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Boundary<T> {
    /// Below every value: the lower end of a range with no lower limit.
    NegInf,
    /// Just below `v`: the upper end of `x < v`.
    Below(T),
    /// Exactly at `v`: a closed end, as in `x <= v` or `x >= v`.
    At(T),
    /// Just above `v`: the lower end of `x > v`.
    Above(T),
    /// Above every value: the upper end of a range with no upper limit.
    PosInf,
}

impl<T> Boundary<T> {
    /// The value this boundary sits at or next to, or `None` for an infinity.
    ///
    /// ```
    /// use rangecomp::Boundary;
    ///
    /// assert_eq!(Boundary::Below(5).value(), Some(&5));
    /// assert_eq!(Boundary::<i32>::PosInf.value(), None);
    /// ```
    pub fn value(&self) -> Option<&T> {
        match self {
            Below(v) | At(v) | Above(v) => Some(v),
            NegInf | PosInf => None,
        }
    }

    /// Borrows the value, giving a `Boundary<&T>` on the same side.
    ///
    /// Useful for comparing or printing without cloning the value.
    ///
    /// ```
    /// use rangecomp::Boundary;
    ///
    /// let b = Boundary::At("a".to_string());
    /// assert_eq!(b.as_ref(), Boundary::At(&"a".to_string()));
    /// ```
    pub fn as_ref(&self) -> Boundary<&T> {
        match self {
            NegInf => NegInf,
            Below(v) => Below(v),
            At(v) => At(v),
            Above(v) => Above(v),
            PosInf => PosInf,
        }
    }

    /// Converts the value with `f`, keeping the side it sits on.
    ///
    /// For example, this turns a `Boundary<i64>` into a `Boundary<String>` for
    /// display. If `f` does not preserve order, ranges built from the results
    /// may not mean what you expect.
    ///
    /// ```
    /// use rangecomp::Boundary;
    ///
    /// assert_eq!(Boundary::Below(5).map(|v| v * 10), Boundary::Below(50));
    /// assert_eq!(Boundary::<i32>::PosInf.map(|v| v * 10), Boundary::PosInf);
    /// ```
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Boundary<U> {
        match self {
            NegInf => NegInf,
            Below(v) => Below(f(v)),
            At(v) => At(f(v)),
            Above(v) => Above(f(v)),
            PosInf => PosInf,
        }
    }

    /// Position among boundaries that share a value.
    fn rank(&self) -> u8 {
        match self {
            NegInf => 0,
            Below(_) => 1,
            At(_) => 2,
            Above(_) => 3,
            PosInf => 4,
        }
    }

    /// The lower bound of the values just above this upper bound.
    ///
    /// Upper bounds are never `Above`; see [`AnyRange::new`](crate::AnyRange::new).
    pub(crate) fn start_after(self) -> Self {
        match self {
            Below(v) => At(v),
            At(v) => Above(v),
            b => b,
        }
    }

    /// The upper bound of the values just below this lower bound.
    ///
    /// Lower bounds are never `Below`; see [`AnyRange::new`](crate::AnyRange::new).
    pub(crate) fn end_before(self) -> Self {
        match self {
            At(v) => Below(v),
            Above(v) => At(v),
            b => b,
        }
    }
}

impl<T: Ord> Boundary<T> {
    /// Orders this boundary against the value `v`, as if comparing with `At(v)`.
    pub(crate) fn cmp_value(&self, v: &T) -> Ordering {
        match self {
            NegInf => Ordering::Less,
            Below(b) => b.cmp(v).then(Ordering::Less),
            At(b) => b.cmp(v),
            Above(b) => b.cmp(v).then(Ordering::Greater),
            PosInf => Ordering::Greater,
        }
    }

    /// Whether a range ending at `self` and one starting at `lower` leave
    /// no value between them.
    pub(crate) fn touches(&self, lower: &Self) -> bool {
        lower <= self || matches!((self, lower), (Below(a), At(b)) | (At(a), Above(b)) if a == b)
    }
}

impl<T: Ord> Ord for Boundary<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.value(), other.value()) {
            (Some(a), Some(b)) => a.cmp(b).then_with(|| self.rank().cmp(&other.rank())),
            _ => self.rank().cmp(&other.rank()),
        }
    }
}

impl<T: Ord> PartialOrd for Boundary<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::Boundary::{self, Above, At, Below, NegInf, PosInf};
    use std::cmp::Ordering;

    #[test]
    fn total_order() {
        let line = [
            NegInf,
            Below(1),
            At(1),
            Above(1),
            Below(2),
            At(2),
            Above(2),
            PosInf,
        ];
        for (i, a) in line.iter().enumerate() {
            for (j, b) in line.iter().enumerate() {
                assert_eq!(a.cmp(b), i.cmp(&j), "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn infinities() {
        let neg: Boundary<i32> = NegInf;
        assert_eq!(neg.cmp(&NegInf), Ordering::Equal);
        assert!(NegInf < Below(i32::MIN));
        assert!(Above(i32::MAX) < PosInf);
        assert_eq!(neg.value(), None);
    }

    #[test]
    fn compare_with_value() {
        assert_eq!(Below(5).cmp_value(&5), Ordering::Less);
        assert_eq!(At(5).cmp_value(&5), Ordering::Equal);
        assert_eq!(Above(5).cmp_value(&5), Ordering::Greater);
        assert_eq!(Above(4).cmp_value(&5), Ordering::Less);
        assert_eq!(NegInf.cmp_value(&5), Ordering::Less);
        assert_eq!(PosInf.cmp_value(&5), Ordering::Greater);
    }

    #[test]
    fn touching() {
        // `x < 5` then `x >= 5`, and `x <= 5` then `x > 5`: nothing missing.
        assert!(Below(5).touches(&At(5)));
        assert!(At(5).touches(&Above(5)));
        // `x < 5` then `x > 5`: 5 itself is missing.
        assert!(!Below(5).touches(&Above(5)));
        // Overlapping or sharing a point.
        assert!(At(5).touches(&At(5)));
        assert!(At(6).touches(&At(5)));
        // A gap of values.
        assert!(!At(5).touches(&At(6)));
        assert!(!Below(5).touches(&At(6)));
        assert!(Boundary::<i32>::PosInf.touches(&PosInf));
    }

    #[test]
    fn flip_sides() {
        assert_eq!(Below(5).start_after(), At(5));
        assert_eq!(At(5).start_after(), Above(5));
        assert_eq!(At(5).end_before(), Below(5));
        assert_eq!(Above(5).end_before(), At(5));
        assert_eq!(Boundary::<i32>::PosInf.start_after(), PosInf);
        assert_eq!(Boundary::<i32>::NegInf.end_before(), NegInf);
    }
}
