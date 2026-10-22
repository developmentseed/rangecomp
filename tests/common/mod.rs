//! Strategies shared by the property tests.

use proptest::prelude::*;
use rangecomp::Boundary::{self, Above, At, Below, NegInf, PosInf};

/// A boundary at an even number from 0 to 20, or an infinity. Tests probe
/// every integer, so the odd numbers stand in for values between bounds.
pub fn boundary() -> impl Strategy<Value = Boundary<i32>> {
    let v = (0..=10).prop_map(|v| v * 2);
    prop_oneof![
        1 => Just(NegInf),
        1 => Just(PosInf),
        3 => v.clone().prop_map(Below),
        3 => v.clone().prop_map(At),
        3 => v.prop_map(Above),
    ]
}

/// Up to four pairs of bounds, which may be backwards or overlap.
pub fn bounds() -> impl Strategy<Value = Vec<(Boundary<i32>, Boundary<i32>)>> {
    prop::collection::vec((boundary(), boundary()), 0..5)
}
