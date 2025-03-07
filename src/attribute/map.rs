use std::{
	collections::{HashMap, hash_map::Entry},
	sync::Arc,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{
	attribute::{Attribute, AttributeInstance, AttributeModifier},
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

	dependencies: HashMap<A, Box<[A]>>,
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
			dependencies: HashMap::new(),
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

			let a = attr.dependencies().iter().cloned().cloned().collect();
			self.dependencies.insert(attribute, a);
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

	// fn mark_dependents_dirty

	fn get_mut(&mut self, attribute: A) -> Option<&mut AttributeInstance<A, M, V>> {
		let entry = self.attributes.entry(attribute);

		match entry {
			Entry::Occupied(e) => {
				let attr = e.into_mut();
				attr.set_dirty();
				Some(attr)
			}
			Entry::Vacant(e) => self
				.supplier
				.create_instance(e.key())
				.map(|attr| e.insert(attr)),
		}
	}
}

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

	fn has_attribute(&self, attribute: &A) -> bool {
		self.instances.contains_key(attribute)
	}

	fn value(&self, attribute: &A, attributes: &AttributeMap<A, M, V>) -> Option<V> {
		self.instances
			.get(attribute)
			.map(|attr| attr.compute_value(attributes))
	}
	fn base_value(&self, attribute: &A) -> Option<V> {
		self.instances.get(attribute).map(|attr| attr.base_value())
	}
}

// impl<A, M> From<HashMap<A, Attribute>> for AttributeSupplier<A, M>
// where
// 	A: PartialEq + Eq + Hash,
// {
// 	fn from(attributes: HashMap<A, Attribute>) -> Self {
// 		AttributeSupplier {
// 			attributes: attributes
// 				.into_iter()
// 				.map(|(k, v)| (k, Arc::new(v)))
// 				.collect(),
// 			instances: HashMap::new()
// 		}
// 	}
// }

// pub struct AttributeGuard<'a, A, M>
// where
// 	A: PartialEq + Eq + Hash + SerdeSupport,
// 	M: PartialEq + Eq + Hash + SerdeSupport,
// {
// 	lock: &'a mut AttributeMap<A, M>,
// }

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
