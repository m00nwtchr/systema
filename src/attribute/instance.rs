use std::hash::Hash;

use parking_lot::Mutex;

use crate::{
	attribute::{
		Attribute,
		map::AttributeMap,
		modifier::{AttributeModifier, Op, Value},
	},
	prelude::Operation,
	util_traits::{Key, Number},
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[derive(Clone)]
pub struct AttributeInstance<A, M, V = f32, O = Operation>
where
	A: Key + Hash,
	M: Key,
	V: Number + 'static,
	O: Op<V>,
{
	// #[cfg_attr(feature = "serde", serde(skip))]
	attribute: Attribute<V>,
	// #[cfg_attr(feature = "serde", serde(skip))]
	modifiers: Vec<(M, AttributeModifier<A, V, O>)>,

	raw_value: V,
	#[cfg_attr(feature = "serde", serde(skip))]
	cached_value: Mutex<Option<V>>,
}

impl<A, M, V, O> Clone for AttributeInstance<A, M, V, O>
where
	A: Key + Hash,
	M: Key,
	V: Number + 'static,
	O: Op<V>,
{
	fn clone(&self) -> Self {
		Self {
			attribute: self.attribute.clone(),
			modifiers: self.modifiers.clone(),
			raw_value: self.raw_value,
			cached_value: Mutex::new(*self.cached_value.lock()),
		}
	}
}

impl<A, M, V, O> AttributeInstance<A, M, V, O>
where
	A: Key + Hash,
	M: Key,
	V: Number + 'static,
	O: Op<V>,
{
	pub fn new(attribute: Attribute<V>) -> Self {
		let raw_value = attribute.default_value();
		Self {
			attribute,
			modifiers: Vec::new(),
			raw_value,
			cached_value: Mutex::new(None),
		}
	}

	pub fn builder(attribute: Attribute<V>) -> AttributeBuilder<A, M, V, O> {
		AttributeBuilder {
			attribute,
			modifiers: Vec::new(),
		}
	}

	pub fn raw_value(&self) -> V {
		self.raw_value
	}

	pub(super) fn mark_dirty(&self) {
		self.cached_value.lock().take();
	}

	pub fn set_raw_value(&mut self, value: V) {
		if value != self.raw_value {
			self.raw_value = value;
			self.mark_dirty();
		}
	}

	pub(super) fn compute_value(&self, attributes: &AttributeMap<A, M, V, O>, base: bool) -> V {
		let mut value = self.raw_value;

		for (_, modifier) in &self.modifiers {
			if !base || modifier.base {
				value = Self::apply_modifier(value, modifier, attributes);
			}
		}

		self.attribute.sanitize_value(value)
	}

	fn apply_modifier(
		value: V,
		modifier: &AttributeModifier<A, V, O>,
		attributes: &AttributeMap<A, M, V, O>,
	) -> V {
		let mod_val = match &modifier.value {
			Value::Value(val) => *val,
			Value::Attribute(attr) => attributes.value(attr).unwrap_or_default(),
		};

		modifier.op.apply(value, mod_val)
	}

	pub fn base_value(&self, attributes: &AttributeMap<A, M, V, O>) -> V {
		self.compute_value(attributes, true)
	}

	pub fn value(&self, attributes: &AttributeMap<A, M, V, O>) -> V {
		let mut cached_value = self.cached_value.lock();
		match *cached_value {
			None => {
				let val = self.compute_value(attributes, false);
				cached_value.replace(val);
				val
			}
			Some(val) => val,
		}
	}

	pub fn has_modifier(&self, modifier: &M) -> bool {
		self.modifiers.iter().any(|(m, _)| modifier.eq(m))
	}
	pub fn modifier(&self, modifier: &M) -> Option<&AttributeModifier<A, V, O>> {
		self.modifiers
			.iter()
			.find(|(m, _)| modifier.eq(m))
			.map(|(_, v)| v)
	}

	pub fn add_modifier(&mut self, id: M, modifier: AttributeModifier<A, V, O>) {
		self.modifiers.push((id, modifier));
		self.mark_dirty();
	}

	pub fn remove_modifier(&mut self, id: &M) -> bool {
		let l1 = self.modifiers.len();
		self.modifiers.retain(|(m, _)| id.ne(m));
		if self.modifiers.len() == l1 {
			false
		} else {
			self.mark_dirty();
			true
		}
	}

	pub fn depends_on(&self, attr: &A) -> bool {
		self.modifiers
			.iter()
			.any(|(_, modifier)| modifier.value.is_attribute(attr))
	}
}

impl<A, M, V, O> From<Attribute<V>> for AttributeInstance<A, M, V, O>
where
	A: Key + Hash,
	M: Key,
	V: Number + 'static,
	O: Op<V>,
{
	fn from(value: Attribute<V>) -> Self {
		Self::new(value)
	}
}

impl<A, M, V, O> Default for AttributeInstance<A, M, V, O>
where
	M: Key,
	A: Key + Hash,
	V: Default + Number,
	O: Op<V>,
{
	fn default() -> Self {
		AttributeInstance::new(Attribute::Value(V::default()))
	}
}

#[must_use]
pub struct AttributeBuilder<A, M, V = f32, O = Operation>
where
	A: Key + Hash + 'static,
	M: Key + 'static,
	V: Number + 'static,
	O: Op<V>,
{
	attribute: Attribute<V>,
	modifiers: Vec<(M, AttributeModifier<A, V, O>)>,
}

impl<A, M, V, O> AttributeBuilder<A, M, V, O>
where
	A: Key + Hash,
	M: Key,
	V: Number + 'static,
	O: Op<V>,
{
	pub fn modifier(mut self, key: M, modifier: AttributeModifier<A, V, O>) -> Self {
		self.modifiers.push((key, modifier));
		self
	}

	// pub fn attribute(mut self, attr: A, modifier: AttributeModifier<A, V>) -> Self {
	// 	self.modifier()
	// }
}

impl<A, M, V, O> From<AttributeBuilder<A, M, V, O>> for AttributeInstance<A, M, V, O>
where
	A: Key + Hash,
	M: Key,
	V: Number + 'static,
	O: Op<V>,
{
	fn from(value: AttributeBuilder<A, M, V, O>) -> Self {
		let raw_value = value.attribute.default_value();
		Self {
			attribute: value.attribute,
			modifiers: value.modifiers,
			raw_value,
			..Self::default()
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::attribute::{
		Attribute,
		map::AttributeMap,
		modifier::{AttributeModifier, Operation, Value},
	};

	#[derive(Debug, Clone, PartialEq, Eq, Hash)]
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	struct TestKey(&'static str);

	#[test]
	fn test_attribute_builder_modifier() {
		let attr = Attribute::Value(10);
		let builder: AttributeBuilder<TestKey, TestKey, i32> = AttributeBuilder {
			attribute: attr,
			modifiers: Vec::new(),
		};

		let modifier = AttributeModifier {
			value: Value::Value(5),
			op: Operation::Add,
			base: false,
		};

		let builder = builder.modifier(TestKey("mod1"), modifier.clone());
		assert_eq!(builder.modifiers.len(), 1);
		assert_eq!(builder.modifiers[0].1, modifier);
	}

	#[test]
	fn test_attribute_instance_new() {
		let attr = Attribute::Value(10);
		let instance = AttributeInstance::<TestKey, TestKey, i32>::new(attr);
		assert_eq!(instance.raw_value(), 10);
	}

	#[test]
	fn test_attribute_instance_set_raw_value() {
		let attr = Attribute::Value(10);
		let mut instance = AttributeInstance::<TestKey, TestKey, i32>::new(attr);
		instance.set_raw_value(20);
		assert_eq!(instance.raw_value(), 20);
	}

	#[test]
	fn test_attribute_instance_add_modifier() {
		let attr = Attribute::Value(10);
		let mut instance = AttributeInstance::<TestKey, TestKey, i32>::new(attr);
		let modifier = AttributeModifier {
			value: Value::Value(5),
			op: Operation::Add,
			base: false,
		};
		instance.add_modifier(TestKey("mod1"), modifier);
		assert!(instance.has_modifier(&TestKey("mod1")));
	}

	#[test]
	fn test_attribute_instance_remove_modifier() {
		let attr = Attribute::Value(10);
		let mut instance = AttributeInstance::<TestKey, TestKey, i32>::new(attr);
		let modifier = AttributeModifier {
			value: Value::Value(5),
			op: Operation::Add,
			base: false,
		};
		instance.add_modifier(TestKey("mod1"), modifier);
		assert!(instance.remove_modifier(&TestKey("mod1")));
		assert!(!instance.has_modifier(&TestKey("mod1")));
	}

	#[test]
	fn test_attribute_instance_compute_value_with_multiple_modifiers() {
		let attr = Attribute::Value(10);
		let mut instance = AttributeInstance::<TestKey, TestKey, i32>::new(attr);
		let modifier1 = AttributeModifier {
			value: Value::Value(5),
			op: Operation::Add,
			base: false,
		};
		let modifier2 = AttributeModifier {
			value: Value::Value(3),
			op: Operation::Sub,
			base: false,
		};
		instance.add_modifier(TestKey("mod1"), modifier1);
		instance.add_modifier(TestKey("mod2"), modifier2);
		let attributes = AttributeMap::<TestKey, TestKey, i32>::default();
		assert_eq!(instance.value(&attributes), 12);
	}

	#[test]
	fn test_attribute_instance_depends_on() {
		let attr = Attribute::Value(10);
		let mut instance = AttributeInstance::<TestKey, TestKey, i32>::new(attr);
		let modifier = AttributeModifier {
			value: Value::Attribute(TestKey("dependency")),
			op: Operation::Add,
			base: false,
		};
		instance.add_modifier(TestKey("mod1"), modifier);
		assert!(instance.depends_on(&TestKey("dependency")));
	}

	#[test]
	fn test_attribute_instance_cache() {
		let attr = Attribute::Value(10);
		let instance = AttributeInstance::<TestKey, TestKey, i32>::new(attr);
		let attributes = AttributeMap::<TestKey, TestKey, i32>::default();
		let val1 = instance.value(&attributes);
		let val2 = instance.value(&attributes);
		assert_eq!(val1, val2);
	}
}
