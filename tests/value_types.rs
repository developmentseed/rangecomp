//! Runs the set operations on other value types and checks every answer
//! against the same operations on integers.
//!
//! - `String`, which is not `Copy`: each integer `n` maps to the zero-padded
//!   string of `n + 100`, so string order matches integer order.
//! - `Reverse<i32>`, whose order runs backwards: `n` maps to `Reverse(-n)`.

mod common;

use std::cmp::Reverse;

use common::bounds;
use proptest::prelude::*;
use rangecomp::{AnyRange, MultiRange};

fn text(n: i32) -> String {
    format!("{:03}", n + 100)
}

fn set() -> impl Strategy<Value = MultiRange<i32>> {
    bounds().prop_map(|raw| raw.into_iter().map(AnyRange::from).collect())
}

fn to_text(set: &MultiRange<i32>) -> MultiRange<String> {
    set.iter()
        .map(|r| AnyRange::new(r.lower().map(text), r.upper().map(text)))
        .collect()
}

fn rev(n: i32) -> Reverse<i32> {
    Reverse(-n)
}

fn to_rev(set: &MultiRange<i32>) -> MultiRange<Reverse<i32>> {
    set.iter()
        .map(|r| AnyRange::new(r.lower().map(rev), r.upper().map(rev)))
        .collect()
}

proptest! {
    #[test]
    fn same_as_integers(a in set(), b in set()) {
        let (ta, tb) = (to_text(&a), to_text(&b));
        prop_assert_eq!(&ta | &tb, to_text(&(&a | &b)));
        prop_assert_eq!(&ta & &tb, to_text(&(&a & &b)));
        prop_assert_eq!(&ta - &tb, to_text(&(&a - &b)));
        prop_assert_eq!(&ta ^ &tb, to_text(&(&a ^ &b)));
        prop_assert_eq!(!&ta, to_text(&!&a));
        prop_assert_eq!(ta.clone() | tb.clone(), to_text(&(a.clone() | b.clone())));
        prop_assert_eq!(!ta.clone(), to_text(&!a.clone()));
        prop_assert_eq!(ta.is_subset(&tb), a.is_subset(&b));
        prop_assert_eq!(ta.is_disjoint(&tb), a.is_disjoint(&b));
        prop_assert_eq!(ta.hull(), a.hull().map(|h| to_text(&h.into()).iter().next().cloned().unwrap()));
        for x in -1..=21 {
            prop_assert_eq!(ta.contains(&text(x)), a.contains(&x));
        }
        let mut inserted = ta.clone();
        for r in &tb {
            inserted.insert(r.clone());
        }
        prop_assert_eq!(inserted, &ta | &tb);
    }

    #[test]
    fn reversed_order(a in set(), b in set()) {
        let (ra, rb) = (to_rev(&a), to_rev(&b));
        prop_assert_eq!(&ra | &rb, to_rev(&(&a | &b)));
        prop_assert_eq!(&ra & &rb, to_rev(&(&a & &b)));
        prop_assert_eq!(&ra - &rb, to_rev(&(&a - &b)));
        prop_assert_eq!(&ra ^ &rb, to_rev(&(&a ^ &b)));
        prop_assert_eq!(!&ra, to_rev(&!&a));
        prop_assert_eq!(ra.is_subset(&rb), a.is_subset(&b));
        for x in -1..=21 {
            prop_assert_eq!(ra.contains(&rev(x)), a.contains(&x));
        }
        let mut inserted = ra.clone();
        for r in &rb {
            inserted.insert(*r);
        }
        prop_assert_eq!(inserted, &ra | &rb);
    }
}
