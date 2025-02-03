use std::ops::{Range, RangeInclusive, RangeFull, RangeFrom, RangeTo, RangeToInclusive};
use boundary::Boundary;

/// The RangeComp Trait enables comparing the interaction between Two Ranges
pub trait RangeComp<T: PartialOrd + PartialEq>{
    fn start_boundary(&self) -> Boundary<T>;
    fn end_boundary(&self) -> Boundary<T>;

    fn intersects(&self, other: &Self) -> bool {
        let ls = self.start_boundary();
        let le = self.end_boundary();
        let rs = other.start_boundary();
        let re = other.end_boundary();
        ls <= re && le <= rs
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
    fn meets(&self, other: &Self) -> bool{
        let le = self.end_boundary();
        let rs = other.start_boundary();
        le == rs
    }
    fn starts(&self, other: &Self) -> bool{
        let ls = self.start_boundary();
        let le = self.end_boundary();
        let rs = other.start_boundary();
        let re = other.end_boundary();
        ls == rs && le < re
    }
    fn during(&self, other: &Self) -> bool{
        let ls = self.start_boundary();
        let le = self.end_boundary();
        let rs = other.start_boundary();
        let re = other.end_boundary();
        ls > rs && le < re
    }
    fn finishes(&self, other: &Self) -> bool{
        let ls = self.start_boundary();
        let le = self.end_boundary();
        let rs = other.start_boundary();
        let re = other.end_boundary();
        ls > rs && le == re
    }
    fn equals(&self, other: &Self) -> bool{
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
            _=> unimplemented!("Operator Does Not Exist")
        }
    }
    fn overlappedby(&self, other: &Self) -> bool {
        other.overlaps(self)
    }
    fn disjoint(&self, other: &Self) -> bool {
        !self.intersects(other)
    }
    fn after(&self, other: &Self) -> bool{
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

impl<T: PartialOrd + Copy> RangeComp<T> for Range<T> {
    fn start_boundary(&self) -> Boundary<T>{
        Boundary::EQ(self.start)
    }
    fn end_boundary(&self) -> Boundary<T>{
        Boundary::LT(self.end)
    }
}

impl<T: PartialOrd + Copy> RangeComp<T> for RangeInclusive<T> {
    fn start_boundary(&self) -> Boundary<T>{
        Boundary::EQ(*self.start())
    }
    fn end_boundary(&self) -> Boundary<T>{
        Boundary::EQ(*self.end())
    }
}

impl<T: PartialOrd + Copy> RangeComp<T> for RangeTo<T> {
    fn start_boundary(&self) -> Boundary<T>{
        Boundary::NegativeInfinity
    }
    fn end_boundary(&self) -> Boundary<T>{
        Boundary::LT(self.end)
    }
}

impl<T: PartialOrd + Copy> RangeComp<T> for RangeFrom<T> {
    fn start_boundary(&self) -> Boundary<T>{
        Boundary::EQ(self.start)
    }
    fn end_boundary(&self) -> Boundary<T>{
        Boundary::Infinity
    }
}

impl<T: PartialOrd + Copy> RangeComp<T> for RangeToInclusive<T> {
    fn start_boundary(&self) -> Boundary<T>{
        Boundary::NegativeInfinity
    }
    fn end_boundary(&self) -> Boundary<T>{
        Boundary::EQ(self.end)
    }
}

impl<T: PartialOrd + Copy> RangeComp<T> for RangeFull {
    fn start_boundary(&self) -> Boundary<T>{
        Boundary::NegativeInfinity
    }
    fn end_boundary(&self) -> Boundary<T>{
        Boundary::Infinity
    }
}

impl<T: PartialOrd + Copy> RangeComp<T> for (Boundary<T>, Boundary<T>) {
    fn start_boundary(&self) -> Boundary<T>{
        self.0
    }
    fn end_boundary(&self) -> Boundary<T>{
        self.1
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_recipricols(){
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
    fn test_float_recipricols(){
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
}
