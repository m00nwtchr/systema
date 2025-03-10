use parking_lot::Mutex;

use crate::{
	attribute::{
		Attribute,
		map::AttributeMap,
		modifier::{AttributeModifier, Value},
	},
	util_traits::{Key, Number},
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[derive(Clone)]
pub struct AttributeInstance<A, M, V = f32>
where
	A: Key,
	M: Key,
	V: Number + 'static,
{
	// #[cfg_attr(feature = "serde", serde(skip))]
	attribute: Attribute<V>,
	// #[cfg_attr(feature = "serde", serde(skip))]
	modifiers: Vec<(M, AttributeModifier<A, V>)>,

	raw_value: V,
	#[cfg_attr(feature = "serde", serde(skip))]
	cached_value: Mutex<Option<V>>,
}

impl<A, M, V> Clone for AttributeInstance<A, M, V>
where
	A: Key,
	M: Key,
	V: Number + 'static,
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

impl<A, M, V> AttributeInstance<A, M, V>
where
	M: Key,
	A: Key,
	V: Number + 'static,
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

	pub fn builder(attribute: Attribute<V>) -> AttributeBuilder<A, M, V> {
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

	pub(super) fn compute_value(&self, attributes: &AttributeMap<A, M, V>, base: bool) -> V {
		let mut value = self.raw_value;

		for (_, modifier) in &self.modifiers {
			if !base || modifier.base {
				value = self.apply_modifier(value, modifier, attributes);
			}
		}

		self.attribute.sanitize_value(value)
	}

	fn apply_modifier(
		&self,
		value: V,
		modifier: &AttributeModifier<A, V>,
		attributes: &AttributeMap<A, M, V>,
	) -> V {
		let mod_val = match &modifier.value {
			Value::Value(val) => *val,
			Value::Attribute(attr) => attributes.value(attr).unwrap_or_default(),
		};

		modifier.op.apply(value, mod_val)
	}

	pub fn base_value(&self, attributes: &AttributeMap<A, M, V>) -> V {
		self.compute_value(attributes, true)
	}

	pub fn value(&self, attributes: &AttributeMap<A, M, V>) -> V {
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
	pub fn modifier(&self, modifier: &M) -> Option<&AttributeModifier<A, V>> {
		self.modifiers
			.iter()
			.find(|(m, _)| modifier.eq(m))
			.map(|(_, v)| v)
	}

	pub fn add_modifier(&mut self, id: M, modifier: AttributeModifier<A, V>) {
		self.modifiers.push((id, modifier));
		self.mark_dirty();
	}

	pub fn remove_modifier(&mut self, id: &M) -> bool {
		let l1 = self.modifiers.len();
		self.modifiers.retain(|(m, _)| id.ne(m));
		if self.modifiers.len() != l1 {
			self.mark_dirty();
			true
		} else {
			false
		}
	}

	pub fn depends_on(&self, attr: &A) -> bool {
		self.modifiers
			.iter()
			.any(|(_, modifier)| modifier.value.is_attribute(attr))
	}
}

impl<A, M, V> From<Attribute<V>> for AttributeInstance<A, M, V>
where
	A: Key,
	M: Key,
	V: Number + 'static,
{
	fn from(value: Attribute<V>) -> Self {
		Self::new(value)
	}
}

impl<A, M, V> Default for AttributeInstance<A, M, V>
where
	M: Key,
	A: Key,
	V: Default + Number,
{
	fn default() -> Self {
		AttributeInstance::new(Attribute::Value(V::default()))
	}
}

pub struct AttributeBuilder<A, M, V = f32>
where
	A: Key + 'static,
	M: Key + 'static,
	V: Number + 'static,
{
	attribute: Attribute<V>,
	modifiers: Vec<(M, AttributeModifier<A, V>)>,
}

impl<A, M, V> AttributeBuilder<A, M, V>
where
	A: Key,
	M: Key,
	V: Number + 'static,
{
	pub fn modifier(mut self, key: M, modifier: AttributeModifier<A, V>) -> Self {
		self.modifiers.push((key, modifier));
		self
	}

	// pub fn attribute(mut self, attr: A, modifier: AttributeModifier<A, V>) -> Self {
	// 	self.modifier()
	// }
}

impl<A, M, V> From<AttributeBuilder<A, M, V>> for AttributeInstance<A, M, V>
where
	A: Key,
	M: Key,
	V: Number + 'static,
{
	fn from(value: AttributeBuilder<A, M, V>) -> Self {
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
