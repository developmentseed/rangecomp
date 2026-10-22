//! Checks every set operation against brute-force membership.

mod common;

use common::{boundary, bounds};
use proptest::prelude::*;
use rangecomp::{
    AnyRange,
    Boundary::{self, Above, At, Below, NegInf, PosInf},
    MultiRange,
};

fn domain() -> std::ops::RangeInclusive<i32> {
    -1..=21
}

/// Membership worked out from the bounds directly.
fn admits((lo, hi): &(Boundary<i32>, Boundary<i32>), x: i32) -> bool {
    let above_lo = match *lo {
        NegInf => true,
        Below(v) | At(v) => x >= v,
        Above(v) => x > v,
        PosInf => false,
    };
    let below_hi = match *hi {
        NegInf => false,
        Below(v) => x < v,
        At(v) | Above(v) => x <= v,
        PosInf => true,
    };
    above_lo && below_hi
}

fn oracle(raw: &[(Boundary<i32>, Boundary<i32>)]) -> impl Fn(i32) -> bool + '_ {
    move |x| raw.iter().any(|b| admits(b, x))
}

fn build(raw: &[(Boundary<i32>, Boundary<i32>)]) -> MultiRange<i32> {
    raw.iter().cloned().map(AnyRange::from).collect()
}

fn assert_matches(set: &MultiRange<i32>, expected: impl Fn(i32) -> bool) {
    for x in domain() {
        assert_eq!(set.contains(&x), expected(x), "{x} in {set:?}");
    }
}

fn assert_normalized(set: &MultiRange<i32>) {
    for i in set {
        assert!(!i.is_empty(), "empty range in {set:?}");
        assert!(!matches!(i.lower(), Below(_)), "lower Below in {set:?}");
        assert!(!matches!(i.upper(), Above(_)), "upper Above in {set:?}");
    }
    for pair in set.as_slice().windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let gap = domain().any(|x| a.upper() < &At(x) && &At(x) < b.lower());
        assert!(gap, "{a:?} and {b:?} should have merged");
    }
}

proptest! {
    #[test]
    fn collect_normalizes(raw in bounds()) {
        let set = build(&raw);
        assert_normalized(&set);
        assert_matches(&set, oracle(&raw));
        let reversed: MultiRange<i32> = set.iter().rev().cloned().collect();
        prop_assert_eq!(reversed, set);
    }

    #[test]
    fn union(a in bounds(), b in bounds()) {
        let (sa, sb) = (build(&a), build(&b));
        let u = sa.union(&sb);
        assert_normalized(&u);
        assert_matches(&u, |x| oracle(&a)(x) || oracle(&b)(x));
        prop_assert_eq!(&u, &sb.union(&sa));
        prop_assert_eq!(&(&sa | &sb), &u);
        prop_assert_eq!(&(sa | sb), &u);
    }

    #[test]
    fn insert_and_extend(a in bounds(), b in bounds()) {
        let (sa, sb) = (build(&a), build(&b));
        let mut inserted = sa.clone();
        for r in b.iter().cloned() {
            inserted.insert(AnyRange::from(r));
        }
        assert_normalized(&inserted);
        prop_assert_eq!(&inserted, &sa.union(&sb));
        let mut extended = sa;
        extended.extend(sb);
        prop_assert_eq!(&extended, &inserted);
    }

    #[test]
    fn intersection(a in bounds(), b in bounds()) {
        let (sa, sb) = (build(&a), build(&b));
        let i = sa.intersection(&sb);
        assert_normalized(&i);
        assert_matches(&i, |x| oracle(&a)(x) && oracle(&b)(x));
        prop_assert_eq!(&i, &sb.intersection(&sa));
        prop_assert_eq!(&(sa & sb), &i);
    }

    #[test]
    fn complement(a in bounds()) {
        let sa = build(&a);
        let c = sa.complement();
        assert_normalized(&c);
        assert_matches(&c, |x| !oracle(&a)(x));
        prop_assert_eq!(&c.complement(), &sa);
        prop_assert!(sa.union(&c).is_full());
        prop_assert!(sa.intersection(&c).is_empty());
        prop_assert_eq!(!sa, c);
    }

    #[test]
    fn difference(a in bounds(), b in bounds()) {
        let (sa, sb) = (build(&a), build(&b));
        let d = sa.difference(&sb);
        assert_normalized(&d);
        assert_matches(&d, |x| oracle(&a)(x) && !oracle(&b)(x));
        prop_assert_eq!(sa - sb, d);
    }

    #[test]
    fn symmetric_difference(a in bounds(), b in bounds()) {
        let (sa, sb) = (build(&a), build(&b));
        let x = sa.symmetric_difference(&sb);
        assert_normalized(&x);
        assert_matches(&x, |v| oracle(&a)(v) != oracle(&b)(v));
        prop_assert_eq!(sa ^ sb, x);
    }

    #[test]
    fn subset(a in bounds(), b in bounds()) {
        let (sa, sb) = (build(&a), build(&b));
        let expected = domain().all(|x| !oracle(&a)(x) || oracle(&b)(x));
        prop_assert_eq!(sa.is_subset(&sb), expected);
        prop_assert_eq!(sb.is_superset(&sa), expected);
        prop_assert!(sa.intersection(&sb).is_subset(&sa));
        prop_assert!(sa.is_subset(&sa.union(&sb)));
    }

    #[test]
    fn disjoint(a in bounds(), b in bounds()) {
        let expected = !domain().any(|x| oracle(&a)(x) && oracle(&b)(x));
        prop_assert_eq!(build(&a).is_disjoint(&build(&b)), expected);
    }

    #[test]
    fn empty_and_full(a in bounds()) {
        let sa = build(&a);
        prop_assert_eq!(sa.is_empty(), !domain().any(oracle(&a)));
        prop_assert_eq!(sa.is_full(), domain().all(oracle(&a)));
    }

    #[test]
    fn hull(a in bounds()) {
        let sa = build(&a);
        match sa.hull() {
            None => prop_assert!(sa.is_empty()),
            Some(h) => {
                let hull = MultiRange::from(h);
                prop_assert!(sa.is_subset(&hull));
                // Nothing outside the set lies beyond its first and last values.
                let gaps = hull.difference(&sa);
                prop_assert!(gaps.iter().all(|g| sa.iter().any(|r| r.upper() < g.lower()) && sa.iter().any(|r| g.upper() < r.lower())));
            }
        }
    }

    #[test]
    fn points(a in bounds()) {
        let sa = build(&a);
        prop_assert_eq!(sa.points().is_some(), sa.iter().all(|r| r.as_point().is_some()));
        if let Some(points) = sa.points() {
            assert_matches(&sa, |x| points.contains(&&x));
        }
    }

    #[test]
    fn to_bounds_round_trip(a in (boundary(), boundary())) {
        let r = AnyRange::from(a);
        match r.to_bounds() {
            Some((lo, hi)) => prop_assert_eq!(AnyRange::from((lo.cloned(), hi.cloned())), r),
            None => prop_assert!(r.is_empty()),
        }
    }

    #[test]
    fn de_morgan(a in bounds(), b in bounds()) {
        let (sa, sb) = (build(&a), build(&b));
        prop_assert_eq!(
            sa.union(&sb).complement(),
            sa.complement().intersection(&sb.complement())
        );
        prop_assert_eq!(
            sa.intersection(&sb).complement(),
            sa.complement().union(&sb.complement())
        );
    }

    #[test]
    fn range_equality_is_membership(a in (boundary(), boundary()), b in (boundary(), boundary())) {
        let same = domain().all(|x| admits(&a, x) == admits(&b, x));
        prop_assert_eq!(AnyRange::from(a) == AnyRange::from(b), same);
    }

    #[test]
    fn allen_relations(a in (boundary(), boundary()), b in (boundary(), boundary())) {
        let (x, y) = (AnyRange::from(a), AnyRange::from(b));
        prop_assert_eq!(
            x.intersects(&y),
            domain().any(|v| admits(&a, v) && admits(&b, v))
        );

        let relations = [
            x.before(&y), x.meets(&y), x.overlaps(&y), x.starts(&y), x.during(&y),
            x.finishes(&y), x.equals(&y), x.after(&y), x.met_by(&y),
            x.overlapped_by(&y), x.started_by(&y), x.encloses(&y), x.finished_by(&y),
        ];
        let proper = |i: &AnyRange<i32>| !i.is_empty() && i.lower() < i.upper();
        if proper(&x) && proper(&y) {
            prop_assert_eq!(relations.iter().filter(|r| **r).count(), 1, "{:?} {:?}", x, y);
        }
        if x.is_empty() || y.is_empty() {
            prop_assert!(relations.iter().all(|r| !r));
        }

        // `before` and `meets` from membership alone: every value of `x` lies
        // below every value of `y`, with or without a value between them.
        // Doubling the bounds leaves at least three probes inside any overlap,
        // so an overlap can't pass for ranges that share a single end point.
        let wide = |(lo, hi): &(Boundary<i32>, Boundary<i32>)| (lo.map(|v| v * 2), hi.map(|v| v * 2));
        let (a, b) = (wide(&a), wide(&b));
        let xs: Vec<i32> = (-4..=44).filter(|v| admits(&a, *v)).collect();
        let ys: Vec<i32> = (-4..=44).filter(|v| admits(&b, *v)).collect();
        let ordered = !xs.is_empty() && !ys.is_empty()
            && xs.iter().all(|p| ys.iter().all(|q| p <= q));
        let gap = (-4..=44).any(|g| xs.iter().all(|p| *p < g) && ys.iter().all(|q| g < *q));
        prop_assert_eq!(x.before(&y), ordered && gap);
        prop_assert_eq!(x.meets(&y), ordered && !gap);
    }
}
