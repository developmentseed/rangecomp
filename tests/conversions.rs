//! Every way of writing a range or a set, and what it turns into.

use std::ops::Bound::{self, Excluded, Included, Unbounded};

use rangecomp::{
    AnyRange,
    Boundary::{self, Above, At, Below, NegInf, PosInf},
    MultiRange,
};

fn bounds<T>(r: AnyRange<T>) -> (Boundary<T>, Boundary<T>) {
    r.into_bounds()
}

#[test]
fn std_range_expressions() {
    assert_eq!(bounds((1..5).into()), (At(1), Below(5)));
    assert_eq!(bounds((1..=5).into()), (At(1), At(5)));
    assert_eq!(bounds((1..).into()), (At(1), PosInf));
    assert_eq!(bounds((..5).into()), (NegInf, Below(5)));
    assert_eq!(bounds((..=5).into()), (NegInf, At(5)));
    assert_eq!(bounds::<i32>((..).into()), (NegInf, PosInf));
}

#[test]
fn std_ranges_match_constructors() {
    assert_eq!(AnyRange::from(..5), AnyRange::less_than(5));
    assert_eq!(AnyRange::from(..=5), AnyRange::at_most(5));
    assert_eq!(AnyRange::from(5..), AnyRange::at_least(5));
    assert_eq!(AnyRange::from(1..=5), AnyRange::between(1, 5));
    assert_eq!(AnyRange::from(5..=5), AnyRange::point(5));
    assert_eq!(AnyRange::<i32>::from(..), AnyRange::full());
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn reversed_and_empty_std_ranges() {
    assert_eq!(AnyRange::from(5..5), AnyRange::empty());
    assert_eq!(AnyRange::from(5..1), AnyRange::empty());
    assert_eq!(AnyRange::from(5..=1), AnyRange::empty());
    assert!(MultiRange::from(5..1).is_empty());
}

#[test]
fn std_ranges_of_other_types() {
    let words = AnyRange::from("b".to_string().."d".to_string());
    assert!(words.contains(&"c".to_string()));

    let borrowed = AnyRange::from("b".."d");
    assert!(borrowed.contains(&"c") && !borrowed.contains(&"d"));

    let chars = MultiRange::from(['a'..='c', 'x'..='z']);
    assert!(chars.contains(&'b') && !chars.contains(&'m'));

    let floats_by_bits = AnyRange::from(1u64..);
    assert!(floats_by_bits.contains(&u64::MAX));
}

#[test]
fn bound_pairs() {
    let cases = [
        ((Included(1), Included(5)), (At(1), At(5))),
        ((Included(1), Excluded(5)), (At(1), Below(5))),
        ((Included(1), Unbounded), (At(1), PosInf)),
        ((Excluded(1), Included(5)), (Above(1), At(5))),
        ((Excluded(1), Excluded(5)), (Above(1), Below(5))),
        ((Excluded(1), Unbounded), (Above(1), PosInf)),
        ((Unbounded, Included(5)), (NegInf, At(5))),
        ((Unbounded, Excluded(5)), (NegInf, Below(5))),
        ((Unbounded, Unbounded), (NegInf, PosInf)),
    ];
    for (pair, expected) in cases {
        assert_eq!(bounds(pair.into()), expected, "{pair:?}");
        assert_eq!(MultiRange::from(pair), AnyRange::from(pair).into());
    }
    assert!(AnyRange::from((Excluded(5), Excluded(5))).is_empty());
}

#[test]
fn boundary_pairs() {
    assert_eq!(
        AnyRange::from((Above(1), Below(5))),
        AnyRange::new(Above(1), Below(5))
    );
    // Equivalent bounds are stored in one form.
    assert_eq!(bounds((Below(1), Above(5)).into()), (At(1), At(5)));
    assert_eq!(AnyRange::from((At(5), Below(5))), AnyRange::empty());
    assert_eq!(
        MultiRange::from((NegInf, PosInf)),
        MultiRange::<i32>::full()
    );
}

#[test]
fn to_bounds_round_trip() {
    let ranges = [
        AnyRange::from(1..5),
        AnyRange::from(1..=5),
        AnyRange::from(1..),
        AnyRange::from(..5),
        AnyRange::from(..=5),
        AnyRange::from(..),
        AnyRange::greater_than(1),
        AnyRange::new(Above(1), At(5)),
    ];
    for r in ranges {
        let (lo, hi) = r.to_bounds().unwrap();
        let back: (Bound<i32>, Bound<i32>) = (lo.cloned(), hi.cloned());
        assert_eq!(AnyRange::from(back), r);
    }
    assert_eq!(AnyRange::<i32>::empty().to_bounds(), None);
}

#[test]
fn sets_from_single_ranges() {
    assert_eq!(
        MultiRange::from(1..5),
        MultiRange::from(AnyRange::from(1..5))
    );
    assert_eq!(
        MultiRange::from(1..=5),
        MultiRange::from(AnyRange::between(1, 5))
    );
    assert_eq!(
        MultiRange::from(1..),
        MultiRange::from(AnyRange::at_least(1))
    );
    assert_eq!(
        MultiRange::from(..5),
        MultiRange::from(AnyRange::less_than(5))
    );
    assert_eq!(
        MultiRange::from(..=5),
        MultiRange::from(AnyRange::at_most(5))
    );
    assert_eq!(MultiRange::<i32>::from(..), MultiRange::full());
    let s: MultiRange<i32> = (1..5).into();
    assert_eq!(s.as_slice(), [AnyRange::from(1..5)]);
}

#[test]
fn sets_from_arrays() {
    assert_eq!(
        MultiRange::from([7..9, 1..3]).as_slice(),
        [AnyRange::from(1..3), AnyRange::from(7..9)]
    );
    let in_list = MultiRange::from([5, 1, 3].map(AnyRange::point));
    assert_eq!(in_list.points(), Some(vec![&1, &3, &5]));
    let mixed = MultiRange::from([AnyRange::less_than(0), AnyRange::from(10..)]);
    assert_eq!(mixed.complement(), MultiRange::from(0..10));
    let none: [AnyRange<i32>; 0] = [];
    assert!(MultiRange::from(none).is_empty());
}

#[test]
fn sets_from_iterators() {
    let collected: MultiRange<i32> = (0..10).step_by(2).map(AnyRange::point).collect();
    assert_eq!(collected.as_slice().len(), 5);

    let mut extended = MultiRange::default();
    extended.extend([AnyRange::from(3..5), AnyRange::from(1..3)]);
    assert_eq!(extended, MultiRange::from(1..5));

    let mut inserted = MultiRange::empty();
    inserted.insert(AnyRange::from(1..3));
    inserted.insert(AnyRange::from(3..5));
    assert_eq!(inserted, extended);

    let ranges: Vec<AnyRange<i32>> = extended.clone().into_iter().collect();
    assert_eq!(ranges, [AnyRange::from(1..5)]);
    assert_eq!((&extended).into_iter().count(), 1);
}

#[test]
fn operators() {
    let a = MultiRange::from(0..10);
    let b = MultiRange::from(5..15);
    assert_eq!(&a | &b, MultiRange::from(0..15));
    assert_eq!(&a & &b, MultiRange::from(5..10));
    assert_eq!(&a - &b, MultiRange::from(0..5));
    assert_eq!(
        !&a,
        MultiRange::from([AnyRange::from(..0), AnyRange::from(10..)])
    );
    assert_eq!(a.clone() | b.clone(), &a | &b);
    assert_eq!(a.clone() & b.clone(), &a & &b);
    assert_eq!(a.clone() - b.clone(), &a - &b);
    assert_eq!(!a.clone(), !&a);
}

#[test]
fn borrowed_and_mapped_bounds() {
    let r = AnyRange::from("a".to_string().."c".to_string());
    assert_eq!(r.as_ref().lower(), &At(&"a".to_string()));
    assert_eq!(At(2).map(|v| v * 10), At(20));
    assert_eq!(Below(2).as_ref(), Below(&2));
}

// These types are newer than the MSRV, and only exist when `core_range` is set.
#[cfg(core_range)]
#[allow(clippy::incompatible_msrv)]
#[test]
fn core_range_types() {
    use std::range;
    assert_eq!(
        AnyRange::from(range::Range::from(1..5)),
        AnyRange::from(1..5)
    );
    assert_eq!(
        AnyRange::from(range::RangeInclusive::from(1..=5)),
        AnyRange::from(1..=5)
    );
    assert_eq!(
        AnyRange::from(range::RangeFrom::from(1..)),
        AnyRange::from(1..)
    );
    assert_eq!(
        AnyRange::from(range::RangeToInclusive::from(..=5)),
        AnyRange::from(..=5)
    );
    assert_eq!(
        MultiRange::from(range::Range::from(1..5)),
        MultiRange::from(1..5)
    );
    assert_eq!(
        MultiRange::from(range::RangeInclusive::from(1..=5)),
        MultiRange::from(1..=5)
    );
    assert_eq!(
        MultiRange::from(range::RangeFrom::from(1..)),
        MultiRange::from(1..)
    );
    assert_eq!(
        MultiRange::from(range::RangeToInclusive::from(..=5)),
        MultiRange::from(..=5)
    );
}

#[test]
fn display() {
    let cases = [
        (MultiRange::from(1..5), "[1, 5)"),
        (MultiRange::from(1..=5), "[1, 5]"),
        (MultiRange::from((Excluded(1), Included(5))), "(1, 5]"),
        (MultiRange::from((Excluded(1), Excluded(5))), "(1, 5)"),
        (MultiRange::from(..5), "(-inf, 5)"),
        (MultiRange::from(5..), "[5, inf)"),
        (MultiRange::from(..), "(-inf, inf)"),
        (MultiRange::from(AnyRange::point(5)), "{5}"),
        (MultiRange::empty(), "∅"),
        (
            !MultiRange::from(AnyRange::point(5)),
            "(-inf, 5) ∪ (5, inf)",
        ),
    ];
    for (set, text) in cases {
        assert_eq!(set.to_string(), text);
    }
    assert_eq!(AnyRange::from("a".."b").to_string(), "[a, b)");
}
