use rangecomp::*;

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
