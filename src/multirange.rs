use std::cmp::{max, min};

use crate::{AnyRange, Boundary, RangeComp};
use std::ops::Deref;
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct MultiRange<T: PartialEq + PartialOrd + std::fmt::Debug + Copy + Clone>(pub Vec<AnyRange<T>>);

impl<T: PartialEq + PartialOrd + std::fmt::Debug + Copy> Deref for MultiRange<T> {
    type Target = Vec<AnyRange<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: PartialEq + PartialOrd + std::fmt::Debug + Copy> std::ops::DerefMut for MultiRange<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: PartialOrd + PartialEq + std::fmt::Debug + Copy> From<AnyRange<T>> for MultiRange<T> {
    fn from(v: AnyRange<T>) -> MultiRange<T> {
        MultiRange(vec!(v))
    }
}



impl<T: PartialOrd + PartialEq + Copy + Ord + std::fmt::Debug> AnyRange<T> {
    pub fn union(self, other: AnyRange<T>) -> MultiRange<T> {
        if self.before(&other) {
            MultiRange(vec!(self, other))
        } else if self.after(&other) {
            MultiRange(vec!(other, self))
        } else {
            let start_boundary: Boundary<T> = min(self.start_boundary, other.start_boundary);
            let end_boundary: Boundary<T> = max(self.end_boundary, other.end_boundary);
            AnyRange{start_boundary, end_boundary}.into()
        }
    }
    pub fn intersection(self, other: AnyRange<T>) -> Option<AnyRange<T>>{
        if self.disjoint(&other) {
            None
        } else {
            let start_boundary: Boundary<T> = max(self.start_boundary, other.start_boundary);
            let end_boundary: Boundary<T> = min(self.end_boundary, other.end_boundary);
            Some(AnyRange{start_boundary, end_boundary})
        }
    }
}

impl<T: std::cmp::PartialOrd + Ord + std::fmt::Debug + Copy> From<Vec<AnyRange<T>>> for MultiRange<T> {
    fn from(v: Vec<AnyRange<T>>) -> MultiRange<T> {
        MultiRange(v)
    }
}


/// Cleans a dirty `MultiRange` by sorting the ranges and merging any intersecting ranges into a single range.
///
/// This function ensures that the `MultiRange` is in a normalized state where all ranges are sorted
/// and no two ranges overlap.
///
/// # Examples
///
/// ```rust
/// use rangecomp::*;
///
/// let mut ranges = MultiRange::new();
/// ranges.push((3..7).into());
/// ranges.push((10..15).into());
/// ranges.push((1..5).into());
/// ranges.push((1..2).into());
/// ranges.push((3..4).into());
/// ranges.push((13..40).into());
/// dbg!(&ranges);
/// ranges.clean_union();
/// dbg!(&ranges);
///
/// assert_eq!(ranges.len(), 2);
/// assert_eq!(ranges[0], AnyRange::from(1..7));
/// assert_eq!(ranges[1], AnyRange::from(10..40));
/// ```
/// ```rust
/// use rangecomp::*;
///
/// let mut ranges = MultiRange::new();
/// ranges.push((3..7).into());
/// ranges.push((1..5).into());
/// ranges.push((3..4).into());
/// ranges.push((1..40).into());
/// dbg!(&ranges);
/// ranges.clean_intersection();
/// dbg!(&ranges);
///
/// assert_eq!(ranges.len(), 1);
/// assert_eq!(ranges[0], AnyRange::from(3..4));
/// ```
impl<T: PartialOrd + PartialEq + Copy + Ord + std::fmt::Debug> MultiRange<T>  {
    pub fn new() -> MultiRange<T> {
        vec!().into()
    }
    pub fn start(self) -> Boundary<T> {
        self[0].start_boundary
    }
    pub fn end(self) -> Boundary<T> {
        self[self.len()-1].end_boundary
    }
    pub fn push(&mut self, other: AnyRange<T>){
        self.0.push(other);
        self.sort();
    }
    /// Clean a dirty MultiRange by sorting and unioning any intersecting ranges
    ///
    pub fn clean_union(&mut self) {
        let mut new = MultiRange::new();
        self.sort();

        let end = self.clone().end();
        let mut workstart: Boundary<T>;
        let mut workend: Boundary<T>;
        let mut nextstart: Boundary<T>;
        let mut nextend: Boundary<T>;

        workstart = self[0].start_boundary;
        workend = self[0].end_boundary;
        dbg!(&self.len());
        for idx in 1..self.len() {
            dbg!(&idx);
            if workend == end {
                new.push((workstart, workend).into());
                dbg!("short circuit", &new);
                break
            }
            nextstart = self[idx].start_boundary;
            nextend = self[idx].end_boundary;
            dbg!(&workstart, &workend, &nextstart, &nextend);
            if workend >= nextstart {
                workend = max(nextend, workend);
                dbg!("workend >= next start, moving to next");
            } else {
                dbg!("workend<nextstart block");
                new.push((workstart, workend).into());
                workstart = nextstart;
                workend = nextend;
                dbg!("workend<nextstart", &workstart, &workend, &new);
            }
        }
        new.push((workstart, workend).into());
        *self = new.clone();
        dbg!(&self);
    }
    pub fn union(&mut self, other: AnyRange<T>) {
        self.push(other);
        self.clean_union();

    }
    // Clean a dirty MultiRange by sorting and intersectioning any intersecting ranges
    ///
    pub fn clean_intersection(&mut self) {
        let mut new = MultiRange::new();
        self.sort();

        let mut working = self[0];
        dbg!(&self.len());
        for idx in 0..self.len()-1 {
            let next = self[idx+1];
            let check = working.intersection(next);
            match check {
                None => {
                    dbg!(&idx, "breaking");
                    break
                },
                Some(v) => {
                    working = v;
                    new = working.into();
                    dbg!(&idx, "adding intersection", &working, &new);
                }
            }
        }

        *self = new.clone();
        dbg!(&self);
    }
    pub fn intersection(&mut self, other: AnyRange<T>){
        self.push(other);
        self.clean_intersection();
    }
}
