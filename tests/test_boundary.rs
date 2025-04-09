use rangecomp::*;
use num_traits::Float;

#[test]
fn test_boundary_ordering() {
    let b1 = Boundary::EQ(1);
    let b2 = Boundary::LT(2);
    let b3 = Boundary::Infinity;
    assert!(b1 < b2);
    assert!(b2 < b3);
}

#[test]
fn test_float_casts() {
    let inf: f64 = Float::infinity();
    let ninf: f64 = Float::neg_infinity();
    let v = 1.234;
    let infb: Boundary<f64> = inf.into();
    let ninfb: Boundary<f64> = ninf.into();
    let vb: Boundary<f64> = v.into();
    assert_eq!(infb, Boundary::Infinity);
    assert_eq!(ninfb, Boundary::NegativeInfinity);
    assert_eq!(vb, Boundary::EQ(v));
}

#[test]
fn test_str_to_boundary_casts() {
    let b = "inf".parse::<Boundary<f64>>().unwrap();
    assert_eq!(b, Boundary::Infinity);

    let b = "-inf".parse::<Boundary<f64>>().unwrap();
    assert_eq!(b, Boundary::NegativeInfinity);

    let b = ">0".parse::<Boundary<f64>>().unwrap();
    assert_eq!(b, Boundary::GT(0.0));

    let b = "<0".parse::<Boundary<f64>>().unwrap();
    assert_eq!(b, Boundary::LT(0.0));
}
