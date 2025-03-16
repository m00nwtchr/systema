use std::{
	collections::{HashMap, hash_map::Entry},
	hash::Hash,
	sync::Arc,
};

use crate::{
	attribute::{
		instance::AttributeInstance,
		modifier::{AttributeModifier, Op},
		supplier::AttributeSupplier,
	},
	prelude::Operation,
	util_traits::{Key, Number},
};

#[cfg(feature = "serde")]
fn sp_default<A: Key + Hash, M: Key, V: Number + 'static, O: Op<V>>()
-> Option<Arc<AttributeSupplier<A, M, V, O>>> {
	None
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, derive_more::Debug)]
pub struct AttributeMap<A, M, V = f32, O = Operation>
where
	A: Key + Hash + 'static,
	M: Key + 'static,
	V: Number + 'static,
	O: Op<V>,
{
	#[cfg_attr(feature = "serde", serde(skip, default = "sp_default"))]
	supplier: Option<Arc<AttributeSupplier<A, M, V, O>>>,
	attributes: HashMap<A, AttributeInstance<A, M, V, O>>,
}

impl<A, M, V, O> AttributeMap<A, M, V, O>
where
	A: Key + Hash,
	M: Key,
	V: Number + 'static,
	O: Op<V>,
{
	#[must_use]
	pub fn new(supplier: Arc<AttributeSupplier<A, M, V, O>>) -> Self {
		AttributeMap {
			supplier: Some(supplier),
			attributes: HashMap::new(),
		}
	}

	pub fn has_attribute(&self, attribute: &A) -> bool {
		self.attributes.contains_key(attribute)
	}
	pub fn has_modifier(&self, attribute: &A, modifier: &M) -> bool {
		self.attributes
			.get(attribute)
			.and_then(|attr| attr.modifier(modifier))
			.is_some()
	}

	pub fn add_modifier(
		&mut self,
		attribute: &A,
		modifier: M,
		instance: AttributeModifier<A, V, O>,
	) -> &mut Self {
		if let Some(attr) = self.get_mut(attribute.clone()) {
			attr.add_modifier(modifier, instance);
			self.mark_dependents_dirty(attribute);
		}

		self
	}

	pub fn remove_modifier(&mut self, attribute: &A, modifier: &M) {
		if let Some(attr) = self.attributes.get_mut(attribute) {
			attr.remove_modifier(modifier);
			self.mark_dependents_dirty(attribute);
		}
	}

	pub fn remove_modifiers(&mut self, modifier: &M) {
		let v: Box<[A]> = self
			.attributes
			.iter_mut()
			.filter_map(|(id, attr)| {
				if attr.remove_modifier(modifier) {
					Some(id.clone())
				} else {
					None
				}
			})
			.collect();

		for id in v {
			self.mark_dependents_dirty(&id);
		}
	}

	pub fn set_raw_value(&mut self, attribute: &A, value: V) {
		if let Some(attr) = self.get_mut(attribute.clone()) {
			attr.set_raw_value(value);
			self.mark_dependents_dirty(attribute);
		}
	}

	pub fn value(&self, attribute: &A) -> Option<V> {
		self.attributes
			.get(attribute)
			.map(|attr| attr.value(self))
			.or_else(|| {
				self.supplier
					.as_ref()
					.and_then(|s| s.value(attribute, self))
			})
	}
	pub fn base_value(&self, attribute: &A) -> Option<V> {
		self.attributes
			.get(attribute)
			.map(|attr| attr.base_value(self))
			.or_else(|| {
				self.supplier
					.as_ref()
					.and_then(|s| s.base_value(attribute, self))
			})
	}

	fn mark_dependents_dirty(&self, id: &A) {
		for (id, attr) in self
			.attributes
			.iter()
			.filter(|(_, attr)| attr.depends_on(id))
		{
			attr.mark_dirty();
			self.mark_dependents_dirty(id);
		}
	}

	fn get_mut(&mut self, attribute: A) -> Option<&mut AttributeInstance<A, M, V, O>> {
		let entry = self.attributes.entry(attribute);

		match entry {
			Entry::Occupied(e) => {
				let attr = e.into_mut();
				attr.mark_dirty();
				Some(attr)
			}
			Entry::Vacant(e) => self
				.supplier
				.as_ref()
				.and_then(|s| s.create_instance(e.key()))
				.map(|attr| e.insert(attr)),
		}
	}
}

impl<A, M, V, O> Default for AttributeMap<A, M, V, O>
where
	A: Key + Hash,
	M: Key,
	V: Number + 'static,
	O: Op<V>,
{
	fn default() -> Self {
		AttributeMap {
			supplier: None,
			attributes: HashMap::new(),
		}
	}
}

#[cfg(test)]
mod tests {
	use once_cell::sync::Lazy;

	use super::*;
	use crate::prelude::{Attribute, Operation};

	#[derive(Debug, Clone, PartialEq, Eq, Hash)]
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	enum TestAttribute {
		Strength,
		Agility,
	}

	#[derive(Debug, Clone, PartialEq, Eq, Hash)]
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	enum TestModifier {
		Buff,
	}

	type MockSupplier = AttributeSupplier<TestAttribute, TestModifier, f32>;
	type MockMap = AttributeMap<TestAttribute, TestModifier, f32>;

	static ATTRIBUTES: Lazy<Arc<MockSupplier>> = Lazy::new(|| {
		Arc::new(
			MockSupplier::builder()
				.add(
					TestAttribute::Strength,
					AttributeInstance::builder(Attribute::Value(1.0)).modifier(
						TestModifier::Buff,
						AttributeModifier::new(1.0, Operation::Add),
					),
				)
				.add(TestAttribute::Agility, Attribute::Value(2.0))
				.build(),
		)
	});

	#[test]
	fn test_new() {
		let map: MockMap = AttributeMap::new(ATTRIBUTES.clone());
		assert!(map.supplier.is_some());
		assert!(map.attributes.is_empty());
	}

	#[test]
	fn test_default() {
		let map: MockMap = AttributeMap::default();
		assert!(map.supplier.is_none());
		assert!(map.attributes.is_empty());
	}

	#[test]
	fn test_has_attribute() {
		let mut map: MockMap = AttributeMap::new(ATTRIBUTES.clone());
		let attr = TestAttribute::Strength;

		assert!(!map.has_attribute(&attr));
		map.set_raw_value(&attr, 10.0);
		assert!(map.has_attribute(&attr));
	}

	#[test]
	fn test_add_and_remove_modifier() {
		let mut map: MockMap = AttributeMap::new(ATTRIBUTES.clone());
		let attr = TestAttribute::Strength;
		let modifier = TestModifier::Buff;
		let mod_instance = AttributeModifier::new(5.0, Operation::Add);

		assert!(!map.has_modifier(&attr, &modifier));

		map.add_modifier(&attr, modifier.clone(), mod_instance);
		assert!(map.has_modifier(&attr, &modifier));

		map.remove_modifier(&attr, &modifier);
		assert!(!map.has_modifier(&attr, &modifier));
	}

	#[test]
	fn test_set_raw_value() {
		let mut map: MockMap = AttributeMap::new(ATTRIBUTES.clone());
		let attr = TestAttribute::Agility;

		map.set_raw_value(&attr, 15.0);
		assert_eq!(map.base_value(&attr), Some(15.0));
	}

	#[test]
	fn test_value_computation() {
		let map: MockMap = AttributeMap::new(ATTRIBUTES.clone());

		let value = map.value(&TestAttribute::Strength);
		assert_eq!(value, Some(2.0));

		let base_value = map.base_value(&TestAttribute::Strength);
		assert_eq!(base_value, Some(1.0));
	}

	#[test]
	fn test_remove_modifiers() {
		let mut map: MockMap = AttributeMap::new(ATTRIBUTES.clone());
		let attr1 = TestAttribute::Strength;
		let attr2 = TestAttribute::Agility;
		let modifier = TestModifier::Buff;
		let mod_instance = AttributeModifier::new(5.0, Operation::Add);

		map.add_modifier(&attr1, modifier.clone(), mod_instance.clone());
		map.add_modifier(&attr2, modifier.clone(), mod_instance);

		assert!(map.has_modifier(&attr1, &modifier));
		assert!(map.has_modifier(&attr2, &modifier));

		map.remove_modifiers(&modifier);

		assert!(!map.has_modifier(&attr1, &modifier));
		assert!(!map.has_modifier(&attr2, &modifier));
	}
}
