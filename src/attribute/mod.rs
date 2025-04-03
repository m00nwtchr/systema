use std::cmp::Ordering;

use crate::util_traits::Number;

pub mod instance;
pub mod map;
pub mod modifier;
pub mod supplier;

pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
	// value is NaN or less than min
	match value.partial_cmp(&min) {
		None | Some(Ordering::Less) => min,
		_ => {
			if value > max {
				max
			} else {
				value
			}
		}
	}
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub enum Attribute<V = f32>
where
	V: Number + 'static,
{
	Value(V),
	Ranged(V, V, V),
	Derived,
}

impl<V> Attribute<V>
where
	V: Number + 'static,
{
	pub fn default_value(&self) -> V {
		match self {
			Self::Ranged(d, _, _) | Self::Value(d) => *d,
			Self::Derived => V::default(),
		}
	}

	pub fn sanitize_value(&self, value: V) -> V {
		match self {
			Self::Ranged(_, min, max) => clamp(value, *min, *max),
			_ => value,
		}
	}
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
	use super::*;

	#[test]
	fn test_clamp_within_range() {
		assert_eq!(clamp(5, 1, 10), 5);
	}

	#[test]
	fn test_clamp_below_min() {
		assert_eq!(clamp(0, 1, 10), 1);
	}

	#[test]
	fn test_clamp_above_max() {
		assert_eq!(clamp(15, 1, 10), 10);
	}

	#[test]
	fn test_clamp_float_nan() {
		let nan: f32 = f32::NAN;
		assert_eq!(1.0, clamp(nan, 1.0, 10.0)); // Maybe should pass the NaN instead?
	}

	#[test]
	fn test_attribute_default_value() {
		let attr_value = Attribute::Value(42);
		assert_eq!(attr_value.default_value(), 42);

		let attr_ranged = Attribute::Ranged(5, 1, 10);
		assert_eq!(attr_ranged.default_value(), 5);

		let attr_derived: Attribute<i32> = Attribute::Derived;
		assert_eq!(attr_derived.default_value(), 0);
	}

	#[test]
	fn test_attribute_sanitize_value() {
		let attr_value = Attribute::Value(42);
		assert_eq!(attr_value.sanitize_value(50), 50);

		let attr_ranged = Attribute::Ranged(5, 1, 10);
		assert_eq!(attr_ranged.sanitize_value(0), 1);
		assert_eq!(attr_ranged.sanitize_value(7), 7);
		assert_eq!(attr_ranged.sanitize_value(15), 10);

		let attr_derived = Attribute::Derived;
		assert_eq!(attr_derived.sanitize_value(99), 99);
	}
}
