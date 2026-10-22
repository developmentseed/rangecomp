use rangecomp::{
    AnyRange,
    Boundary::{Above, At},
};

#[test]
fn allen_closed() {
    let a = AnyRange::between(1, 5);
    assert!(a.before(&AnyRange::between(6, 9)));
    assert!(a.meets(&AnyRange::between(5, 9)));
    assert!(a.overlaps(&AnyRange::between(3, 9)));
    assert!(a.starts(&AnyRange::between(1, 9)));
    assert!(a.during(&AnyRange::between(0, 9)));
    assert!(a.finishes(&AnyRange::between(0, 5)));
    assert!(a.equals(&AnyRange::between(1, 5)));
    assert!(a.after(&AnyRange::between(-3, 0)));
    assert!(a.met_by(&AnyRange::between(-3, 1)));
    assert!(a.overlapped_by(&AnyRange::between(0, 3)));
    assert!(a.started_by(&AnyRange::between(1, 3)));
    assert!(a.encloses(&AnyRange::between(2, 3)));
    assert!(a.finished_by(&AnyRange::between(3, 5)));
}

#[test]
fn allen_open_ends() {
    // No value between `< 5` and `>= 5`, or between `<= 5` and `> 5`.
    assert!(AnyRange::from(1..5).meets(&AnyRange::from(5..9)));
    assert!(AnyRange::between(1, 5).meets(&AnyRange::new(Above(5), At(9))));
    // 5 lies between `< 5` and `> 5`.
    assert!(AnyRange::from(1..5).before(&AnyRange::new(Above(5), At(9))));
    // Starting at 5 versus just above it.
    assert!(AnyRange::between(5, 7).overlaps(&AnyRange::new(Above(5), At(9))));
    assert!(!AnyRange::between(5, 7).starts(&AnyRange::new(Above(5), At(9))));
}

#[test]
fn allen_unbounded() {
    assert!(AnyRange::less_than(5).meets(&AnyRange::at_least(5)));
    assert!(AnyRange::at_most(5).starts(&AnyRange::full()));
    assert!(AnyRange::at_least(5).finishes(&AnyRange::full()));
    assert!(AnyRange::<i32>::full().equals(&AnyRange::full()));
}

#[test]
fn intersects_and_disjoint() {
    assert!(AnyRange::between(1, 5).intersects(&AnyRange::between(5, 9)));
    assert!(AnyRange::from(1..5).disjoint(&AnyRange::from(5..9)));
    assert!(AnyRange::from(1..10).intersects(&AnyRange::from(5..15)));
    let empty = AnyRange::from(5..5);
    assert!(empty.disjoint(&AnyRange::full()));
    assert!(!empty.equals(&empty));
}

#[test]
fn strings() {
    let a = AnyRange::from("a".to_string().."c".to_string());
    let b = AnyRange::from("a".to_string().."b".to_string());
    assert!(a.started_by(&b));
    assert!(b.starts(&a));
}
