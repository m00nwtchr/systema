use std::{collections::HashMap, sync::Arc};

use crate::{
	attribute::{Attribute, AttributeInstance, AttributeModifier, map::AttributeMap},
	util_traits::{Key, Number},
};

pub struct AttributeSupplierBuilder<A, M, V>
where
	A: Key + 'static,
	M: Key + 'static,
	V: Number + 'static,
{
	// attributes: HashMap<A, Arc<Attribute>>,
	instances: HashMap<A, AttributeInstance<A, M, V>>,
}

impl<A, M, V> AttributeSupplierBuilder<A, M, V>
where
	A: Key + 'static,
	M: Key + 'static,
	V: Number + 'static,
{
	pub fn build(self) -> AttributeSupplier<A, M, V> {
		AttributeSupplier {
			// attributes: self.attributes,
			instances: self.instances,
		}
	}

	pub fn add<I: Into<A>>(mut self, id: I, attribute: Attribute<V>) -> Self {
		self.instances
			.insert(id.into(), AttributeInstance::new(Arc::new(attribute)));
		self
	}

	pub fn create<I: Into<A>>(self, id: I, attribute: Attribute<V>) -> AttributeBuilder<A, M, V> {
		AttributeBuilder {
			asb: self,
			id: id.into(),
			attribute: Arc::new(attribute),
			modifiers: HashMap::new(),
		}
	}

	pub fn add_instance<I: Into<A>>(
		mut self,
		id: I,
		attribute: AttributeInstance<A, M, V>,
	) -> Self {
		self.instances.insert(id.into(), attribute);
		self
	}
}

pub struct AttributeSupplier<A, M, V = f32>
where
	A: Key,
	M: Key,
	V: Number + 'static,
{
	// attributes: HashMap<A, Arc<Attribute>>,
	instances: HashMap<A, AttributeInstance<A, M, V>>,
}

impl<A, M, V> AttributeSupplier<A, M, V>
where
	A: Key,
	M: Key,
	V: Number + 'static,
{
	pub fn builder() -> AttributeSupplierBuilder<A, M, V> {
		AttributeSupplierBuilder {
			// attributes: HashMap::new(),
			instances: HashMap::new(),
		}
	}

	pub fn create_instance(&self, attribute: &A) -> Option<AttributeInstance<A, M, V>> {
		self.instances.get(attribute).cloned()
	}

	pub(crate) fn has_attribute(&self, attribute: &A) -> bool {
		self.instances.contains_key(attribute)
	}

	pub(crate) fn value(&self, attribute: &A, attributes: &AttributeMap<A, M, V>) -> Option<V> {
		self.instances
			.get(attribute)
			.map(|attr| attr.compute_value(attributes))
	}
	pub(crate) fn base_value(&self, attribute: &A) -> Option<V> {
		self.instances.get(attribute).map(|attr| attr.base_value())
	}
}

pub struct AttributeBuilder<A, M, V = f32>
where
	A: Key + 'static,
	M: Key + 'static,
	V: Number + 'static,
{
	asb: AttributeSupplierBuilder<A, M, V>,
	id: A,
	attribute: Arc<Attribute<V>>,
	modifiers: HashMap<M, AttributeModifier<A, V>>,
}

impl<A, M, V> AttributeBuilder<A, M, V>
where
	A: Key,
	M: Key,
	V: Number + 'static,
{
	pub fn modifier(mut self, key: M, modifier: AttributeModifier<A, V>) -> Self {
		self.modifiers.insert(key, modifier);
		self
	}

	pub fn insert(self) -> AttributeSupplierBuilder<A, M, V> {
		let mut ai = AttributeInstance::new(self.attribute);
		ai.modifiers = self.modifiers;

		self.asb.add_instance(self.id, ai)
	}
}
