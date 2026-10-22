//! `Discrete` impls that break the trait's rules give wrong answers, but they
//! must never panic, loop forever, or produce a set that breaks its
//! invariants.

use rangecomp::{AnyRange, Discrete, MultiRange};

/// Checks that the ranges are sorted, non-empty and apart.
fn assert_valid<T: Ord + std::fmt::Debug>(set: &MultiRange<T>) {
    for r in set {
        assert!(!r.is_empty(), "{set:?}");
    }
    for pair in set.as_slice().windows(2) {
        assert!(pair[0].upper() < pair[1].lower(), "{set:?}");
    }
}

/// `succ` returns the same value instead of the next one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Stuck(u8);

impl Discrete for Stuck {
    const MIN: Option<Self> = Some(Stuck(0));
    const MAX: Option<Self> = Some(Stuck(u8::MAX));

    fn succ(&self) -> Option<Self> {
        Some(*self)
    }

    fn pred(&self) -> Option<Self> {
        self.0.checked_sub(1).map(Stuck)
    }
}

/// `succ` stays at the top instead of returning `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Saturating(u8);

impl Discrete for Saturating {
    const MIN: Option<Self> = Some(Saturating(0));
    const MAX: Option<Self> = Some(Saturating(u8::MAX));

    fn succ(&self) -> Option<Self> {
        Some(Saturating(self.0.saturating_add(1)))
    }

    fn pred(&self) -> Option<Self> {
        self.0.checked_sub(1).map(Saturating)
    }
}

/// `succ` wraps from the top back to 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Wrapping(u8);

impl Discrete for Wrapping {
    const MIN: Option<Self> = Some(Wrapping(0));
    const MAX: Option<Self> = Some(Wrapping(u8::MAX));

    fn succ(&self) -> Option<Self> {
        Some(Wrapping(self.0.wrapping_add(1)))
    }

    fn pred(&self) -> Option<Self> {
        self.0.checked_sub(1).map(Wrapping)
    }
}

/// `pred` returns `None` at 5, as if 5 were the smallest value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct EarlyPred(u8);

impl Discrete for EarlyPred {
    const MIN: Option<Self> = Some(EarlyPred(0));
    const MAX: Option<Self> = Some(EarlyPred(u8::MAX));

    fn succ(&self) -> Option<Self> {
        self.0.checked_add(1).map(EarlyPred)
    }

    fn pred(&self) -> Option<Self> {
        match self.0 {
            5 => None,
            v => v.checked_sub(1).map(EarlyPred),
        }
    }
}

#[test]
fn stuck_succ_does_not_loop() {
    let set = MultiRange::from(AnyRange::between(Stuck(3), Stuck(9)));
    let values: Vec<_> = set.values().unwrap().take(100).collect();
    assert!(values.len() < 100);
    assert_valid(&set.discrete());
}

#[test]
fn saturating_succ_does_not_gain_values() {
    let above_max = MultiRange::from(AnyRange::greater_than(Saturating(u8::MAX)));
    assert!(above_max.discrete().is_empty());
    let top = MultiRange::from(AnyRange::at_least(Saturating(250)));
    assert_eq!(top.values().unwrap().count(), 6);
}

#[test]
fn wrapping_succ_does_not_wrap() {
    let above_max = MultiRange::from(AnyRange::greater_than(Wrapping(u8::MAX)));
    assert!(above_max.discrete().is_empty());
}

#[test]
fn early_pred_keeps_the_set_valid() {
    let set = MultiRange::from([
        AnyRange::between(EarlyPred(1), EarlyPred(2)),
        AnyRange::from(EarlyPred(5)..EarlyPred(10)),
    ]);
    assert_valid(&set.discrete());
    let values: Vec<_> = set.values().unwrap().collect();
    let mut sorted = values.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(values, sorted);
}
