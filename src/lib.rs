#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod anyrange;
mod boundary;
#[cfg(core_range)]
mod core_range;
mod discrete;
mod multirange;
#[cfg(feature = "serde")]
mod serde;

pub use anyrange::AnyRange;
pub use boundary::Boundary;
pub use discrete::Discrete;
pub use multirange::MultiRange;
