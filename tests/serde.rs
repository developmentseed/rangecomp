#![cfg(feature = "serde")]

mod common;

use common::bounds;
use proptest::prelude::*;
use rangecomp::{AnyRange, Boundary, MultiRange};
use serde_json::json;

proptest! {
    #[test]
    fn round_trips(raw in bounds()) {
        let set: MultiRange<i32> = raw.iter().cloned().map(AnyRange::from).collect();
        let text = serde_json::to_string(&set).unwrap();
        prop_assert_eq!(&serde_json::from_str::<MultiRange<i32>>(&text).unwrap(), &set);

        // Raw, unnormalized input reads back as the normalized set.
        let raw_json: Vec<_> = raw
            .iter()
            .map(|(lower, upper)| json!({"lower": lower, "upper": upper}))
            .collect();
        prop_assert_eq!(serde_json::from_value::<MultiRange<i32>>(json!(raw_json)).unwrap(), set);
    }
}

#[test]
fn boundary_format() {
    assert_eq!(
        serde_json::to_value(Boundary::At(5)).unwrap(),
        json!({"At": 5})
    );
    assert_eq!(
        serde_json::to_value(Boundary::<i32>::NegInf).unwrap(),
        json!("NegInf")
    );
}

#[test]
fn range_format() {
    let r = AnyRange::from(1..5);
    let value = serde_json::to_value(r).unwrap();
    assert_eq!(value, json!({"lower": {"At": 1}, "upper": {"Below": 5}}));
    assert_eq!(serde_json::from_value::<AnyRange<i32>>(value).unwrap(), r);
}

#[test]
fn set_round_trip() {
    let s = MultiRange::from([
        AnyRange::less_than(0),
        AnyRange::point(3),
        AnyRange::from(5..),
    ]);
    let text = serde_json::to_string(&s).unwrap();
    assert_eq!(serde_json::from_str::<MultiRange<i32>>(&text).unwrap(), s);
}

#[test]
fn reading_normalizes() {
    let r: AnyRange<i32> =
        serde_json::from_value(json!({"lower": {"Below": 1}, "upper": {"Above": 5}})).unwrap();
    assert_eq!(r, AnyRange::between(1, 5));

    let empty: AnyRange<i32> =
        serde_json::from_value(json!({"lower": {"At": 5}, "upper": {"At": 1}})).unwrap();
    assert_eq!(empty, AnyRange::empty());

    let s: MultiRange<i32> = serde_json::from_value(json!([
        {"lower": {"At": 5}, "upper": {"Below": 9}},
        {"lower": {"At": 1}, "upper": {"Below": 6}},
        {"lower": {"At": 7}, "upper": {"At": 3}}
    ]))
    .unwrap();
    assert_eq!(s, MultiRange::from(1..9));
}

#[test]
fn bad_input_is_an_error() {
    let bad = [
        json!(null),
        json!([{"lower": "At", "upper": "PosInf"}]),
        json!([{"lower": {"At": "five"}, "upper": "PosInf"}]),
        json!([{"lower": "NegInf", "upper": "PosInf", "extra": 1}]),
        json!([{"lower": "NegInf"}]),
        json!([{"lower": {"Sideways": 5}, "upper": "PosInf"}]),
    ];
    for value in bad {
        assert!(
            serde_json::from_value::<MultiRange<i32>>(value.clone()).is_err(),
            "{value}"
        );
    }
}
