use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use crate::Boundary;

/// AnyRange holds all types of ranges
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct AnyRange<T: PartialOrd + PartialEq + Clone> {
    pub start_boundary: Boundary<T>,
    pub end_boundary: Boundary<T>
}


/// The RangeComp Trait enables comparing the interaction between Two Ranges
pub trait RangeComp<T: PartialOrd + PartialEq> {
    fn start_boundary(&self) -> Boundary<T>;
    fn end_boundary(&self) -> Boundary<T>;

    fn intersects(&self, other: &Self) -> bool {
        let ls = self.start_boundary();
        let le = self.end_boundary();
        let rs = other.start_boundary();
        let re = other.end_boundary();
        ls <= re && le >= rs
    }
    fn overlaps(&self, other: &Self) -> bool {
        let ls = self.start_boundary();
        let le = self.end_boundary();
        let rs = other.start_boundary();
        let re = other.end_boundary();
        ls < re && rs < le && le < re
    }
    fn before(&self, other: &Self) -> bool {
        let le = self.end_boundary();
        let rs = other.start_boundary();
        le < rs
    }
    fn meets(&self, other: &Self) -> bool {
        let le = self.end_boundary();
        let rs = other.start_boundary();
        le == rs
    }
    fn starts(&self, other: &Self) -> bool {
        let ls = self.start_boundary();
        let le = self.end_boundary();
        let rs = other.start_boundary();
        let re = other.end_boundary();
        ls == rs && le < re
    }
    fn during(&self, other: &Self) -> bool {
        let ls = self.start_boundary();
        let le = self.end_boundary();
        let rs = other.start_boundary();
        let re = other.end_boundary();
        ls > rs && le < re
    }
    fn finishes(&self, other: &Self) -> bool {
        let ls = self.start_boundary();
        let le = self.end_boundary();
        let rs = other.start_boundary();
        let re = other.end_boundary();
        ls > rs && le == re
    }
    fn equals(&self, other: &Self) -> bool {
        let ls = self.start_boundary();
        let le = self.end_boundary();
        let rs = other.start_boundary();
        let re = other.end_boundary();
        ls == rs && le == re
    }
    fn op(&self, other: &Self, op: &str) -> bool {
        let lower = op.to_lowercase();
        let op = lower.rsplit('_').next().expect("Operator Does Not Exist");
        match op {
            "intersects" => self.intersects(other),
            "overlaps" => self.overlaps(other),
            "anyinteracts" => self.intersects(other),
            "disjoint" => self.disjoint(other),
            "before" => self.before(other),
            "after" => self.after(other),
            "meeets" => self.meets(other),
            "metby" => self.metby(other),
            "starts" => self.starts(other),
            "startedby" => self.startedby(other),
            "during" => self.during(other),
            "contains" => self.rcontains(other),
            "finishes" => self.finishes(other),
            "finishedby" => self.finishedby(other),
            "equals" => self.equals(other),
            _ => unimplemented!("Operator Does Not Exist")
        }
    }
    fn overlappedby(&self, other: &Self) -> bool {
        other.overlaps(self)
    }
    fn disjoint(&self, other: &Self) -> bool {
        !self.intersects(other)
    }
    fn after(&self, other: &Self) -> bool {
        other.before(self)
    }
    fn metby(&self, other: &Self) -> bool {
        other.meets(self)
    }
    fn startedby(&self, other: &Self) -> bool {
        other.starts(self)
    }
    fn rcontains(&self, other: &Self) -> bool {
        other.during(self)
    }
    fn finishedby(&self, other: &Self) -> bool {
        other.finishes(self)
    }
}


impl<T: PartialOrd + Clone> RangeComp<T> for Range<T> {
    fn start_boundary(&self) -> Boundary<T> {
        Boundary::EQ(self.start.clone())
    }
    fn end_boundary(&self) -> Boundary<T> {
        Boundary::LT(self.end.clone())
    }
}

impl<T: PartialOrd + Clone> RangeComp<T> for RangeInclusive<T> {
    fn start_boundary(&self) -> Boundary<T> {
        Boundary::EQ(self.start().clone())
    }
    fn end_boundary(&self) -> Boundary<T> {
        Boundary::EQ(self.end().clone())
    }
}

impl<T: PartialOrd + Clone> RangeComp<T> for RangeTo<T> {
    fn start_boundary(&self) -> Boundary<T> {
        Boundary::NegativeInfinity
    }
    fn end_boundary(&self) -> Boundary<T> {
        Boundary::LT(self.end.clone())
    }
}

impl<T: PartialOrd + Clone> RangeComp<T> for RangeFrom<T> {
    fn start_boundary(&self) -> Boundary<T> {
        Boundary::EQ(self.start.clone())
    }
    fn end_boundary(&self) -> Boundary<T> {
        Boundary::Infinity
    }
}

impl<T: PartialOrd + Clone> RangeComp<T> for RangeToInclusive<T> {
    fn start_boundary(&self) -> Boundary<T> {
        Boundary::NegativeInfinity
    }
    fn end_boundary(&self) -> Boundary<T> {
        Boundary::EQ(self.end.clone())
    }
}

impl<T: PartialOrd + Clone> RangeComp<T> for RangeFull {
    fn start_boundary(&self) -> Boundary<T> {
        Boundary::NegativeInfinity
    }
    fn end_boundary(&self) -> Boundary<T> {
        Boundary::Infinity
    }
}

impl<T: PartialOrd + Clone> RangeComp<T> for (Boundary<T>, Boundary<T>) {
    fn start_boundary(&self) -> Boundary<T> {
        self.0.clone()
    }
    fn end_boundary(&self) -> Boundary<T> {
        self.1.clone()
    }
}

impl<T: PartialOrd + Clone> RangeComp<T> for AnyRange<T> {
    fn start_boundary(&self) -> Boundary<T> {
        self.start_boundary.clone()
    }
    fn end_boundary(&self) -> Boundary<T> {
        self.end_boundary.clone()
    }
}

impl<T: PartialOrd + Clone> From<(Boundary<T>, Boundary<T>)> for AnyRange<T> {
    fn from(t: (Boundary<T>, Boundary<T>)) -> AnyRange<T> {
        if t.1 > t.0 {
            AnyRange { start_boundary: t.0, end_boundary: t.1 }
        } else {
            AnyRange { start_boundary: t.1, end_boundary: t.0 }
        }
    }
}

impl<T: PartialOrd + Clone> From<Range<T>> for AnyRange<T> {
    fn from(v: Range<T>) -> AnyRange<T> {
        AnyRange { start_boundary: v.start_boundary(), end_boundary: v.end_boundary() }
    }
}

impl<T: PartialOrd + Clone> From<RangeInclusive<T>> for AnyRange<T> {
    fn from(v: RangeInclusive<T>) -> AnyRange<T> {
        AnyRange { start_boundary: v.start_boundary(), end_boundary: v.end_boundary() }
    }
}

impl<T: PartialOrd + Clone> From<RangeTo<T>> for AnyRange<T> {
    fn from(v: RangeTo<T>) -> AnyRange<T> {
        AnyRange { start_boundary: v.start_boundary(), end_boundary: v.end_boundary() }
    }
}

impl<T: PartialOrd + Clone> From<RangeFrom<T>> for AnyRange<T> {
    fn from(v: RangeFrom<T>) -> AnyRange<T> {
        AnyRange { start_boundary: v.start_boundary(), end_boundary: v.end_boundary() }
    }
}

impl<T: PartialOrd + Clone> From<RangeToInclusive<T>> for AnyRange<T> {
    fn from(v: RangeToInclusive<T>) -> AnyRange<T> {
        AnyRange { start_boundary: v.start_boundary(), end_boundary: v.end_boundary() }
    }
}

impl<T: PartialOrd + Clone> From<RangeFull> for AnyRange<T> {
    fn from(v: RangeFull) -> AnyRange<T> {
        AnyRange { start_boundary: v.start_boundary(), end_boundary: v.end_boundary() }
    }
}

impl<T: PartialOrd + PartialEq + Clone> PartialOrd for AnyRange<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.start_boundary.partial_cmp(&other.start_boundary) {
            Some(std::cmp::Ordering::Equal) => self.end_boundary.partial_cmp(&other.end_boundary),
            other => other,
        }
    }
}

impl<T: Ord + Clone> Ord for AnyRange<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.start_boundary.cmp(&other.start_boundary) {
            std::cmp::Ordering::Equal => self.end_boundary.cmp(&other.end_boundary),
            other => other,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_recipricols() {
        let ref l = 1..10;
        let ref r = 5..15;
        assert_eq!(l.starts(r), r.startedby(l));
        assert_eq!(l.meets(r), r.metby(l));
        assert_eq!(l.finishes(r), r.finishes(l));
        assert_eq!(l.after(r), r.before(l));
        assert_eq!(l.rcontains(r), r.during(l));
        assert_eq!(l.overlaps(r), r.overlappedby(l));
    }

    #[test]
    fn test_float_recipricols() {
        let ref l = 1.0..10.0;
        let ref r = 5.0..15.0;
        assert_eq!(l.starts(r), r.startedby(l));
        assert_eq!(l.meets(r), r.metby(l));
        assert_eq!(l.finishes(r), r.finishes(l));
        assert_eq!(l.after(r), r.before(l));
        assert_eq!(l.rcontains(r), r.during(l));
        assert_eq!(l.overlaps(r), r.overlappedby(l));
    }

    #[test]
    fn int_startedby() {
        let ref left = 1..4;
        let ref right = 1..3;
        assert!(left.startedby(right));
    }
    #[test]
    fn float_startedby() {
        let ref left = 1.0..4.7;
        let ref right = 1.0..3.8;
        assert!(left.startedby(right));
    }
    #[test]
    fn jiff_startedby() {
        use jiff::Timestamp;
        let s: Timestamp = "2020-01-01 00:00:00Z".parse().unwrap();
        let e1: Timestamp = "2020-03-01 00:00:00Z".parse().unwrap();
        let e2: Timestamp = "2020-02-01 00:00:00Z".parse().unwrap();
        let ref left = s..e1;
        let ref right = s..e2;
        assert!(left.startedby(right));
    }
    #[test]
    fn text_startedby() {
        let a: String = "a".into();
        let b: String = "b".into();
        let c: String = "c".into();
        let ref left = a.clone()..c;
        let ref right = a.clone()..b;
        assert!(left.startedby(right));
    }
}
