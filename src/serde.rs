//! `Serialize` and `Deserialize` for the range types. Deserializing goes
//! through the normal constructors, so the result is always in normal form.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{AnyRange, Boundary, MultiRange};

#[derive(Serialize)]
struct RangeRef<'a, T> {
    lower: &'a Boundary<T>,
    upper: &'a Boundary<T>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RangeOwned<T> {
    lower: Boundary<T>,
    upper: Boundary<T>,
}

impl<T: Serialize> Serialize for AnyRange<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        RangeRef {
            lower: &self.lower,
            upper: &self.upper,
        }
        .serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de> + Ord> Deserialize<'de> for AnyRange<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let RangeOwned { lower, upper } = RangeOwned::deserialize(deserializer)?;
        Ok(AnyRange::new(lower, upper))
    }
}

impl<T: Serialize> Serialize for MultiRange<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de> + Ord> Deserialize<'de> for MultiRange<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<AnyRange<T>>::deserialize(deserializer).map(|ranges| ranges.into_iter().collect())
    }
}
