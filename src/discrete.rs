/// A type whose values each have an immediate successor and predecessor, such
/// as the integers.
///
/// [`AnyRange::discrete`](crate::AnyRange::discrete) and
/// [`MultiRange::discrete`](crate::MultiRange::discrete) use it to close open
/// bounds and to merge ranges with no value between them, and
/// [`MultiRange::values`](crate::MultiRange::values) uses it to list a set's
/// values.
///
/// It is implemented for the integer types, `char` (skipping the surrogate
/// range `D800..=DFFF`) and `bool`. The `jiff` and `chrono` features add
/// `jiff::civil::Date` and `chrono::NaiveDate`.
///
/// # Implementing
///
/// `succ` returns the smallest greater value and `pred` the largest smaller
/// one. `MIN` must be `Some(m)` exactly when `m.pred()` is `None`, and `MAX`
/// must be `Some(m)` exactly when `m.succ()` is `None`; a type with no first
/// or last value uses `None`.
///
/// An impl that breaks these rules gives wrong answers, but never a set that
/// breaks its own invariants: a `succ` that does not move forward, or a
/// `pred` that does not move back, counts as `None`.
///
/// The orphan rule allows implementing this trait only for your own types, so
/// wrap foreign types in a newtype:
///
/// ```
/// use rangecomp::{AnyRange, Boundary::{Above, Below}, Discrete, MultiRange};
///
/// /// Days counted from an epoch, with no first or last day.
/// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// struct Day(i64);
///
/// impl Discrete for Day {
///     const MIN: Option<Self> = None;
///     const MAX: Option<Self> = None;
///
///     fn succ(&self) -> Option<Self> {
///         Some(Day(self.0 + 1))
///     }
///
///     fn pred(&self) -> Option<Self> {
///         Some(Day(self.0 - 1))
///     }
/// }
///
/// let week = MultiRange::from(AnyRange::new(Above(Day(0)), Below(Day(8))));
/// assert_eq!(week.values().unwrap().count(), 7);
/// assert!(MultiRange::from(AnyRange::greater_than(Day(0))).values().is_none());
/// ```
pub trait Discrete: Ord + Clone {
    /// The smallest value, if the type has one.
    const MIN: Option<Self>;

    /// The largest value, if the type has one.
    const MAX: Option<Self>;

    /// The next value, or `None` for the largest.
    fn succ(&self) -> Option<Self>;

    /// The previous value, or `None` for the smallest.
    fn pred(&self) -> Option<Self>;
}

/// The value after `v`, ignoring a `succ` result that is not greater than `v`.
pub(crate) fn next<T: Discrete>(v: &T) -> Option<T> {
    v.succ().filter(|n| n > v)
}

/// The value before `v`, ignoring a `pred` result that is not less than `v`.
pub(crate) fn prev<T: Discrete>(v: &T) -> Option<T> {
    v.pred().filter(|p| p < v)
}

macro_rules! impl_discrete {
    ($($t:ty),*) => {$(
        impl Discrete for $t {
            const MIN: Option<Self> = Some(<$t>::MIN);
            const MAX: Option<Self> = Some(<$t>::MAX);

            fn succ(&self) -> Option<Self> {
                self.checked_add(1)
            }

            fn pred(&self) -> Option<Self> {
                self.checked_sub(1)
            }
        }
    )*};
}

impl_discrete!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

impl Discrete for char {
    const MIN: Option<Self> = Some('\0');
    const MAX: Option<Self> = Some(char::MAX);

    fn succ(&self) -> Option<Self> {
        match *self {
            '\u{D7FF}' => Some('\u{E000}'),
            c => char::from_u32(c as u32 + 1),
        }
    }

    fn pred(&self) -> Option<Self> {
        match *self {
            '\u{E000}' => Some('\u{D7FF}'),
            c => (c as u32).checked_sub(1).and_then(char::from_u32),
        }
    }
}

impl Discrete for bool {
    const MIN: Option<Self> = Some(false);
    const MAX: Option<Self> = Some(true);

    fn succ(&self) -> Option<Self> {
        if *self {
            None
        } else {
            Some(true)
        }
    }

    fn pred(&self) -> Option<Self> {
        if *self {
            Some(false)
        } else {
            None
        }
    }
}

#[cfg(feature = "jiff")]
impl Discrete for jiff::civil::Date {
    const MIN: Option<Self> = Some(jiff::civil::Date::MIN);
    const MAX: Option<Self> = Some(jiff::civil::Date::MAX);

    fn succ(&self) -> Option<Self> {
        self.tomorrow().ok()
    }

    fn pred(&self) -> Option<Self> {
        self.yesterday().ok()
    }
}

#[cfg(feature = "chrono")]
impl Discrete for chrono::NaiveDate {
    const MIN: Option<Self> = Some(chrono::NaiveDate::MIN);
    const MAX: Option<Self> = Some(chrono::NaiveDate::MAX);

    fn succ(&self) -> Option<Self> {
        self.succ_opt()
    }

    fn pred(&self) -> Option<Self> {
        self.pred_opt()
    }
}

#[cfg(test)]
mod tests {
    use super::Discrete;

    /// Checks the `MIN`/`MAX` contract at the type's edges.
    fn edges_agree<T: Discrete + std::fmt::Debug>() {
        let (min, max) = (T::MIN.unwrap(), T::MAX.unwrap());
        assert_eq!(min.pred(), None);
        assert_eq!(max.succ(), None);
        assert_eq!(min.succ().and_then(|v| v.pred()), Some(min));
        assert_eq!(max.pred().and_then(|v| v.succ()), Some(max));
    }

    #[test]
    fn built_in_edges() {
        edges_agree::<i8>();
        edges_agree::<u64>();
        edges_agree::<i128>();
        edges_agree::<usize>();
        edges_agree::<char>();
        edges_agree::<bool>();
    }

    #[test]
    fn integers() {
        assert_eq!(0i32.pred(), Some(-1));
        assert_eq!(0usize.pred(), None);
    }

    #[test]
    fn chars() {
        assert_eq!('a'.succ(), Some('b'));
        assert_eq!('\u{D7FF}'.succ(), Some('\u{E000}'));
        assert_eq!('\u{E000}'.pred(), Some('\u{D7FF}'));
    }

    #[test]
    fn bools() {
        assert_eq!(false.succ(), Some(true));
        assert_eq!(true.pred(), Some(false));
    }

    #[cfg(feature = "jiff")]
    #[test]
    fn jiff_dates() {
        use jiff::civil::date;
        edges_agree::<jiff::civil::Date>();
        assert_eq!(date(2024, 2, 28).succ(), Some(date(2024, 2, 29)));
        assert_eq!(date(2025, 1, 1).pred(), Some(date(2024, 12, 31)));
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn chrono_dates() {
        use chrono::NaiveDate;
        edges_agree::<NaiveDate>();
        let d = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        // NaiveDate's own deprecated `succ` and `pred` shadow the trait's.
        assert_eq!(Discrete::succ(&d(2024, 2, 28)), Some(d(2024, 2, 29)));
        assert_eq!(Discrete::pred(&d(2025, 1, 1)), Some(d(2024, 12, 31)));
    }
}
