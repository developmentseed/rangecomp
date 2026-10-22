//! Typical uses of the public API.

use rangecomp::{
    AnyRange,
    Boundary::{Above, Below},
    Discrete, MultiRange,
};

fn set<T: Ord + Clone>(ranges: impl IntoIterator<Item = AnyRange<T>>) -> MultiRange<T> {
    ranges.into_iter().collect()
}

#[test]
fn points_and_their_complement() {
    let points = set([1, 3, 5].map(AnyRange::point));
    assert_eq!(points.points(), Some(vec![&1, &3, &5]));

    let others = points.complement();
    assert!(!others.contains(&3) && others.contains(&4));
    assert_eq!(others.as_slice().len(), 4);
    assert!(points.intersection(&others).is_empty());
}

#[test]
fn range_and_points() {
    let range = MultiRange::from(AnyRange::between(2, 4));
    let points = set([1, 3, 5].map(AnyRange::point));
    assert_eq!(range.intersection(&points).points(), Some(vec![&3]));
}

#[test]
fn empty_and_full_results() {
    let gt5 = MultiRange::from(AnyRange::greater_than(5));
    let lt5 = MultiRange::from(AnyRange::less_than(5));
    let le5 = MultiRange::from(AnyRange::at_most(5));
    assert!(gt5.intersection(&lt5).is_empty());
    assert!(!gt5.union(&lt5).is_full());
    assert!(gt5.union(&le5).is_full());
}

#[test]
fn hull_and_holes() {
    // Everything above 0 except 3 and 7.
    let s = set([AnyRange::point(3), AnyRange::point(7)])
        .complement()
        .intersection(&AnyRange::greater_than(0).into());
    let hull = s.hull().unwrap();
    assert_eq!(hull, AnyRange::greater_than(0));
    let holes = MultiRange::from(hull).difference(&s);
    assert_eq!(holes.points(), Some(vec![&3, &7]));
}

#[test]
fn timestamps_as_strings() {
    let day = MultiRange::from(AnyRange::from(
        "2020-01-01".to_string().."2020-01-02".to_string(),
    ));
    assert!(day.contains(&"2020-01-01T12:00:00Z".to_string()));
    assert!(!day.contains(&"2020-01-02".to_string()));
}

#[test]
fn discrete_integers() {
    // Over integers, (4, 7) holds 5 and 6.
    let s =
        MultiRange::from(AnyRange::greater_than(4)).intersection(&AnyRange::less_than(7).into());
    assert_eq!(s.discrete(), AnyRange::between(5, 6).into());
    assert_eq!(
        s.values().map(Iterator::collect::<Vec<_>>),
        Some(vec![5, 6])
    );

    // Over integers, `..=4` and `5..` together cover everything.
    let all = set([AnyRange::at_most(4), AnyRange::at_least(5)]);
    assert!(!all.is_full());
    assert!(all.discrete().is_full());
}

/// Days counted from an epoch, with no first or last day.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Day(i64);

impl Discrete for Day {
    const MIN: Option<Self> = None;
    const MAX: Option<Self> = None;

    fn succ(&self) -> Option<Self> {
        Some(Day(self.0 + 1))
    }

    fn pred(&self) -> Option<Self> {
        Some(Day(self.0 - 1))
    }
}

#[test]
fn discrete_without_edges() {
    let week = MultiRange::from(AnyRange::new(Above(Day(0)), Below(Day(8))));
    assert_eq!(week.values().map(|v| v.count()), Some(7));
    assert!(MultiRange::from(AnyRange::greater_than(Day(0)))
        .values()
        .is_none());
}

#[test]
fn discrete_edges() {
    // `x >= 0` over u8 is everything, and `x > 255` is nothing.
    assert!(MultiRange::from(AnyRange::at_least(0u8))
        .discrete()
        .is_full());
    assert!(MultiRange::from(AnyRange::greater_than(u8::MAX))
        .discrete()
        .is_empty());
    // A set touching the type's edge still lists its values.
    let low = MultiRange::from(AnyRange::at_most(i8::MIN + 1));
    assert_eq!(
        low.values().map(Iterator::collect::<Vec<_>>),
        Some(vec![-128, -127])
    );
}

#[test]
fn booleans() {
    let both = set([AnyRange::point(true), AnyRange::point(false)]);
    assert!(both.discrete().is_full());

    let not_true = MultiRange::from(AnyRange::point(true)).complement();
    let is_false = MultiRange::from(AnyRange::point(false));
    assert_eq!(not_true.discrete(), is_false.discrete());
    assert_eq!(
        not_true.values().map(Iterator::collect::<Vec<_>>),
        Some(vec![false])
    );
}

#[test]
fn chars() {
    let inside = MultiRange::from(AnyRange::new(Above('a'), Below('e')));
    let letters = MultiRange::from(AnyRange::between('a', 'e')).difference(&inside);
    assert_eq!(
        letters.values().map(Iterator::collect::<String>),
        Some("ae".into())
    );
}

#[test]
fn values_are_lazy() {
    let first: Vec<i64> = MultiRange::full().values().unwrap().take(2).collect();
    assert_eq!(first, [i64::MIN, i64::MIN + 1]);
    let last: Vec<u128> = MultiRange::from(AnyRange::at_least(u128::MAX - 1))
        .values()
        .unwrap()
        .collect();
    assert_eq!(last, [u128::MAX - 1, u128::MAX]);
}
