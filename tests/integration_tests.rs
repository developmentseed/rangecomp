use rangecomp::*;
use std::ops::Range;

#[test]
fn test_anyrange_creation() {
    let range: AnyRange<i32> = (Boundary::EQ(1), Boundary::LT(10)).into();
    assert_eq!(range.start_boundary, Boundary::EQ(1));
    assert_eq!(range.end_boundary, Boundary::LT(10));
}

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
fn test_multirange_union() {
    let mut ranges = MultiRange::new();
    ranges.push((1..5).into());
    ranges.push((3..7).into());
    ranges.clean_union();
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], AnyRange::from(1..7));
}

#[test]
fn test_multirange_intersection() {
    let mut ranges = MultiRange::new();
    ranges.push((1..5).into());
    ranges.push((3..7).into());
    ranges.clean_intersection();
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], AnyRange::from(3..5));
}

#[test]
fn test_boundary_ordering() {
    let b1 = Boundary::EQ(1);
    let b2 = Boundary::LT(2);
    let b3 = Boundary::Infinity;
    assert!(b1 < b2);
    assert!(b2 < b3);
}

#[test]
fn test_rangecomp_op() {
    let range1: AnyRange<i32> = (1..10).into();
    let range2: AnyRange<i32> = (5..15).into();
    assert!(range1.op(&range2, "intersects"));
    assert!(range1.op(&range2, "overlaps"));
    assert!(!range1.op(&range2, "disjoint"));
}

#[test]
fn test_multirange_push_and_sort() {
    let mut ranges = MultiRange::new();
    ranges.push((10..15).into());
    ranges.push((1..5).into());
    ranges.push((7..9).into());
    assert_eq!(ranges.len(), 3);
    ranges.sort();
    assert_eq!(ranges[0], AnyRange::from(1..5));
    assert_eq!(ranges[1], AnyRange::from(7..9));
    assert_eq!(ranges[2], AnyRange::from(10..15));
}
