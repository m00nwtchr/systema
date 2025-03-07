use std::collections::{HashMap, hash_map::Entry};

use itertools::Itertools;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{
	attribute::{AttributeInstance, AttributeModifier, Value, supplier::AttributeSupplier},
	util_traits::{Key, Number},
};

#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct AttributeMap<A, M, V = f32>
where
	A: Key + 'static,
	M: Key + 'static,
	V: Number + 'static,
{
	#[cfg_attr(feature = "serde", serde(skip))]
	supplier: &'static AttributeSupplier<A, M, V>,
	attributes: HashMap<A, AttributeInstance<A, M, V>>,
}

impl<A, M, V> AttributeMap<A, M, V>
where
	A: Key,
	M: Key,
	V: Number + 'static,
{
	pub fn new(supplier: &'static AttributeSupplier<A, M, V>) -> Self {
		AttributeMap {
			supplier,
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
		attribute: A,
		modifier: M,
		instance: AttributeModifier<A, V>,
	) -> &mut Self {
		if let Some(attr) = self.get_mut(attribute.clone()) {
			attr.add_modifier(modifier, instance);
		}

		self
	}

	pub fn remove_modifiers(&mut self, modifier: &M) {
		for attr in self.attributes.values_mut() {
			attr.remove_modifier(modifier);
		}
	}

	pub fn set_base_value(&mut self, attribute: A, value: V) {
		if let Some(attr) = self.get_mut(attribute) {
			attr.set_base_value(value)
		}
	}

	pub fn value(&self, attribute: &A) -> Option<V> {
		self.attributes
			.get(attribute)
			.map(|attr| attr.value(self))
			.or_else(|| self.supplier.value(attribute, self))
	}
	pub fn base_value(&self, attribute: &A) -> Option<V> {
		self.attributes
			.get(attribute)
			.map(AttributeInstance::base_value)
			.or_else(|| self.supplier.base_value(attribute))
	}

	fn mark_dependents_dirty(&mut self, id: &A) {
		for attrs in self
			.attributes
			.values_mut()
			.filter(|attr| attr.depends_on(id))
		{
			attrs.mark_dirty();
		}
	}

	fn get_mut(&mut self, attribute: A) -> Option<&mut AttributeInstance<A, M, V>> {
		let entry = self.attributes.entry(attribute);

		match entry {
			Entry::Occupied(e) => {
				let attr = e.into_mut();
				attr.mark_dirty();
				Some(attr)
			}
			Entry::Vacant(e) => self
				.supplier
				.create_instance(e.key())
				.map(|attr| e.insert(attr)),
		}
	}
}
