use crate::util_traits::Number;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
	Add,
	Sub,
}

impl<V: Number + 'static> Op<V> for Operation {
	fn apply(&self, a: V, b: V) -> V {
		match self {
			Self::Add => a + b,
			Self::Sub => a - b,
		}
	}
}

pub trait Op<V>: Clone {
	fn apply(&self, a: V, b: V) -> V;
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Value<A, V> {
	Value(V),
	Attribute(A),
}

impl<A: PartialEq, V> Value<A, V> {
	pub fn is_attribute(&self, attr: &A) -> bool {
		match self {
			Self::Attribute(a) => a.eq(attr),
			_ => false,
		}
	}
}

impl<A, V: Number> From<V> for Value<A, V> {
	fn from(value: V) -> Self {
		Self::Value(value)
	}
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeModifier<A, V: 'static, O: Op<V> = Operation> {
	pub value: Value<A, V>,
	pub op: O,
	pub base: bool,
}

impl<A, V, O: Op<V>> AttributeModifier<A, V, O> {
	pub fn new<I: Into<Value<A, V>>>(value: I, op: O) -> Self {
		Self {
			value: value.into(),
			op,
			base: false,
		}
	}

	pub const fn new_const(value: Value<A, V>, op: O) -> Self {
		Self {
			value,
			op,
			base: false,
		}
	}

	pub fn base(mut self) -> Self {
		self.base = true;
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_value_enum() {
		let value: Value<&str, i32> = Value::Value(42);
		let attr: Value<&str, i32> = Value::Attribute("strength");

		assert!(!value.is_attribute(&"strength"));
		assert!(attr.is_attribute(&"strength"));
	}

	#[test]
	fn test_value_from_impl() {
		let val: Value<&str, i32> = 10.into();

		assert_eq!(val, Value::Value(10));
	}

	#[test]
	fn test_attribute_modifier() {
		let mod1: AttributeModifier<&str, i32> = AttributeModifier::new(5, Operation::Add);
		assert!(!mod1.base);

		let mod2 = mod1.clone().base();
		assert!(mod2.base);

		assert_eq!(mod2.value, Value::Value(5));
	}

	#[test]
	fn test_operations() {
		let add = Operation::Add;
		let sub = Operation::Sub;

		assert_eq!(add.apply(2, 8), 10);
		assert_eq!(sub.apply(5, 3), 2);
	}

	#[test]
	fn test_operation_debug() {
		let add = Operation::Add;
		let sub = Operation::Sub;

		assert_ne!(format!("{add:?} {sub:?}"), "");
	}

	#[test]
	fn test_partial_eq_for_operation() {
		let add_op1 = Operation::Add;
		let add_op2 = Operation::Add;
		let sub_op1 = Operation::Sub;
		let sub_op2 = Operation::Sub;

		assert_eq!(add_op1, add_op2);
		assert_ne!(add_op1, sub_op1);
		assert_eq!(sub_op1, sub_op2);
	}
}
