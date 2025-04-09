use rangecomp::*;
use std::ops::Range;

#[test]
fn test_rangecomp_intersects() {
    let range1: AnyRange<i32> = (1..10).into();
    let range2: AnyRange<i32> = (5..15).into();
    assert!(range1.intersects(&range2));
    assert!(range2.intersects(&range1));
}

#[test]
fn test_rangecomp_disjoint() {
    let range1: AnyRange<i32> = (1..5).into();
    let range2: AnyRange<i32> = (10..15).into();
    assert!(range1.disjoint(&range2));
    assert!(range2.disjoint(&range1));
}

#[test]
fn test_rangecomp_op() {
    let range1: AnyRange<i32> = (1..10).into();
    let range2: AnyRange<i32> = (5..15).into();
    assert!(range1.op(&range2, "intersects"));
    assert!(range1.op(&range2, "overlaps"));
    assert!(!range1.op(&range2, "disjoint"));
}
