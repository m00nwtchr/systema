use std::marker::PhantomData;

use crate::util_traits::Number;

pub trait CloneableFn<V>: Fn(V, V) -> V + Send + Sync {
	fn clone_box<'a>(&self) -> Box<dyn 'a + CloneableFn<V>>
	where
		Self: 'a;
}

impl<F, V> CloneableFn<V> for F
where
	F: Fn(V, V) -> V + Clone + Send + Sync,
{
	fn clone_box<'a>(&self) -> Box<dyn 'a + CloneableFn<V>>
	where
		Self: 'a,
	{
		Box::new(self.clone())
	}
}

impl<'a, V: 'a> Clone for Box<dyn 'a + CloneableFn<V>> {
	fn clone(&self) -> Self {
		(**self).clone_box()
	}
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub enum Operation<V: 'static> {
	Add,
	Sub,
	#[cfg(not(feature = "serde"))]
	Fn(Box<dyn CloneableFn<V>>),
	#[cfg(feature = "serde")]
	Phantom(PhantomData<V>),
}

impl<V: Number + 'static> Operation<V> {
	pub fn apply(&self, a: V, b: V) -> V {
		match self {
			Self::Add => a + b,
			Self::Sub => a - b,
			#[cfg(not(feature = "serde"))]
			Self::Fn(f) => f(a, b),
			#[cfg(feature = "serde")]
			Phantom => panic!(),
		}
	}
}

impl<V: 'static> std::fmt::Debug for Operation<V> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Operation::Add => write!(f, "Operation::Add"),
			Operation::Sub => write!(f, "Operation::Sub"),
			#[cfg(not(feature = "serde"))]
			Operation::Fn(_) => write!(f, "Operation::Fn)"),
			#[cfg(feature = "serde")]
			Phantom => panic!(),
		}
	}
}

impl<V: 'static> PartialEq for Operation<V> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Operation::Add, Operation::Add) => true,
			(Operation::Sub, Operation::Sub) => true,
			#[cfg(not(feature = "serde"))]
			(Operation::Fn(_), Operation::Fn(_)) => false, // Can't compare function pointers meaningfully
			_ => false,
		}
	}
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
pub struct AttributeModifier<A, V: 'static> {
	pub value: Value<A, V>,
	pub op: Operation<V>,
	pub base: bool,
}

impl<A, V> AttributeModifier<A, V> {
	pub fn new<I: Into<Value<A, V>>>(value: I, op: Operation<V>) -> Self {
		Self {
			value: value.into(),
			op,
			base: false,
		}
	}

	pub const fn new_const(value: Value<A, V>, op: Operation<V>) -> Self {
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
	fn test_cloneable_fn() {
		let add_fn: Box<dyn CloneableFn<i32>> = Box::new(|a, b| a + b);
		let cloned_fn = add_fn.clone();
		assert_eq!(add_fn(2, 3), cloned_fn(2, 3));
	}

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
		#[cfg(not(feature = "serde"))]
		let min = Operation::Fn(Box::new(i32::min));
		let add = Operation::Add;
		let sub = Operation::Sub;

		#[cfg(not(feature = "serde"))]
		assert_eq!(min.apply(7, 10), 7);
		assert_eq!(add.apply(2, 8), 10);
		assert_eq!(sub.apply(5, 3), 2);
	}

	#[test]
	fn test_operation_debug() {
		#[cfg(not(feature = "serde"))]
		let min: Operation<i32> = Operation::Fn(Box::new(i32::min));
		let add: Operation<i32> = Operation::Add;
		let sub: Operation<i32> = Operation::Sub;

		assert_ne!(format!("{add:?} {sub:?}"), "");
		#[cfg(not(feature = "serde"))]
		assert_ne!(format!("{min:?}"), "");
	}

	#[test]
	fn test_partial_eq_for_operation() {
		let add_op1: Operation<f32> = Operation::Add;
		let add_op2 = Operation::Add;
		let sub_op1 = Operation::Sub;
		let sub_op2 = Operation::Sub;

		#[cfg(not(feature = "serde"))]
		let fn_op1 = Operation::Fn(Box::new(|a, b| a + b));
		#[cfg(not(feature = "serde"))]
		let fn_op2 = Operation::Fn(Box::new(|a, b| a + b));

		assert_eq!(add_op1, add_op2);
		assert_ne!(add_op1, sub_op1);
		assert_eq!(sub_op1, sub_op2);

		#[cfg(not(feature = "serde"))]
		assert_ne!(fn_op1, sub_op2);
		#[cfg(not(feature = "serde"))]
		assert_ne!(fn_op1, fn_op2);
	}
}
