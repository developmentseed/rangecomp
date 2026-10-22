//! Checks the discrete form against brute-force membership over every `i8`.

use proptest::prelude::*;
use rangecomp::{
    AnyRange,
    Boundary::{self, Above, At, Below, NegInf, PosInf},
    MultiRange,
};

fn value() -> impl Strategy<Value = i8> {
    prop_oneof![Just(i8::MIN), Just(i8::MAX), -6i8..=6]
}

fn boundary() -> impl Strategy<Value = Boundary<i8>> {
    prop_oneof![
        1 => Just(NegInf),
        1 => Just(PosInf),
        3 => value().prop_map(Below),
        3 => value().prop_map(At),
        3 => value().prop_map(Above),
    ]
}

fn set() -> impl Strategy<Value = MultiRange<i8>> {
    prop::collection::vec((boundary(), boundary()), 0..5)
        .prop_map(|raw| raw.into_iter().map(AnyRange::from).collect())
}

fn members(s: &MultiRange<i8>) -> Vec<i8> {
    (i8::MIN..=i8::MAX).filter(|x| s.contains(x)).collect()
}

proptest! {
    #[test]
    fn keeps_membership(s in set()) {
        prop_assert_eq!(members(&s.discrete()), members(&s));
    }

    #[test]
    fn is_canonical(s in set()) {
        let d = s.discrete();
        let from_points: MultiRange<i8> = members(&s).into_iter().map(AnyRange::point).collect();
        prop_assert_eq!(&from_points.discrete(), &d);
        prop_assert_eq!(&d.discrete(), &d);
    }

    #[test]
    fn bounds_are_closed_and_apart(s in set()) {
        let d = s.discrete();
        for r in &d {
            prop_assert!(matches!(r.lower(), NegInf | At(_)), "{:?}", d);
            prop_assert!(matches!(r.upper(), At(_) | PosInf), "{:?}", d);
            prop_assert!(r.lower() != &At(i8::MIN) && r.upper() != &At(i8::MAX), "{:?}", d);
        }
        for pair in d.as_slice().windows(2) {
            if let (At(u), At(l)) = (pair[0].upper(), pair[1].lower()) {
                prop_assert!(i16::from(*l) - i16::from(*u) > 1, "{:?}", d);
            }
        }
    }

    #[test]
    fn values_lists_members(s in set()) {
        let values: Option<Vec<i8>> = s.values().map(Iterator::collect);
        prop_assert_eq!(values, Some(members(&s)));
    }

    #[test]
    fn operations_then_discrete(a in set(), b in set()) {
        let (da, db) = (a.discrete(), b.discrete());
        prop_assert_eq!(a.union(&b).discrete(), da.union(&db).discrete());
        prop_assert_eq!(a.intersection(&b).discrete(), da.intersection(&db).discrete());
        prop_assert_eq!(a.complement().discrete(), da.complement().discrete());
        let subset = members(&a).iter().all(|x| b.contains(x));
        prop_assert_eq!(a.difference(&b).discrete().is_empty(), subset);
    }
}
