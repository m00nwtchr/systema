use crate::attribute::{Attribute, AttributeInstance, AttributeModifier};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use crate::serde_support::SerdeSupport;
#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct AttributeMap<A, M>
where
	A: Clone + PartialEq + Eq + Hash + SerdeSupport + 'static,
	M: Clone + PartialEq + Eq + Hash + SerdeSupport + 'static,
{
	#[cfg_attr(feature = "serde", serde(skip))]
	supplier: &'static AttributeSupplier<A, M>,
	attributes: HashMap<A, AttributeInstance<A, M>>,

	dependencies: HashMap<A, Box<[A]>>,
}

impl<A, M> AttributeMap<A, M>
where
	A: Clone + PartialEq + Eq + Hash + SerdeSupport,
	M: Clone + PartialEq + Eq + Hash + SerdeSupport,
{
	pub fn new(supplier: &'static AttributeSupplier<A, M>) -> Self {
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

	pub fn add_modifier(&mut self, attribute: A, modifier: M, instance: AttributeModifier<A>) {
		if let Some(attr) = self.get_mut(attribute.clone()) {
			attr.add_modifier(modifier, instance);

			let a = attr.dependencies().iter().cloned().cloned().collect();
			self.dependencies.insert(attribute, a);
		}
	}

	pub fn set_base_value(&mut self, attribute: A, value: f32) {
		if let Some(attr) = self.get_mut(attribute) {
			attr.set_base_value(value)
		}
	}

	pub fn value(&self, attribute: &A) -> Option<f32> {
		self.attributes
			.get(attribute)
			.map(|attr| attr.value(self))
			.or_else(|| self.supplier.value(attribute, self))
	}
	pub fn base_value(&self, attribute: &A) -> Option<f32> {
		self.attributes
			.get(attribute)
			.map(AttributeInstance::base_value)
			.or_else(|| self.supplier.base_value(attribute))
	}

	// fn mark_dependents_dirty
	
	fn get_mut(&mut self, attribute: A) -> Option<&mut AttributeInstance<A, M>> {
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

pub struct AttributeSupplierBuilder<A, M>
where
	A: Clone + PartialEq + Eq + Hash + SerdeSupport,
	M: Clone + PartialEq + Eq + Hash + SerdeSupport,
{
	// attributes: HashMap<A, Arc<Attribute>>,
	instances: HashMap<A, AttributeInstance<A, M>>,
}

impl<A, M> AttributeSupplierBuilder<A, M>
where
	A: Clone + PartialEq + Eq + Hash + SerdeSupport,
	M: Clone + PartialEq + Eq + Hash + SerdeSupport,
{
	pub fn build(self) -> AttributeSupplier<A, M> {
		AttributeSupplier {
			// attributes: self.attributes,
			instances: self.instances,
		}
	}

	pub fn add<I: Into<A>>(mut self, id: I, attribute: Attribute) -> Self {
		self.instances
			.insert(id.into(), AttributeInstance::new(Arc::new(attribute)));
		self
	}

	pub fn add_instance<I: Into<A>>(mut self, id: I, attribute: AttributeInstance<A, M>) -> Self {
		self.instances.insert(id.into(), attribute);
		self
	}
}

pub struct AttributeSupplier<A, M>
where
	A: Clone + PartialEq + Eq + Hash + SerdeSupport,
	M: Clone + PartialEq + Eq + Hash + SerdeSupport,
{
	// attributes: HashMap<A, Arc<Attribute>>,
	instances: HashMap<A, AttributeInstance<A, M>>,
}

impl<A, M> AttributeSupplier<A, M>
where
	A: Clone + PartialEq + Eq + Hash + SerdeSupport,
	M: Clone + PartialEq + Eq + Hash + SerdeSupport,
{
	pub fn builder() -> AttributeSupplierBuilder<A, M> {
		AttributeSupplierBuilder {
			// attributes: HashMap::new(),
			instances: HashMap::new(),
		}
	}

	pub fn create_instance(&self, attribute: &A) -> Option<AttributeInstance<A, M>> {
		self.instances.get(attribute).cloned()
	}

	fn has_attribute(&self, attribute: &A) -> bool {
		self.instances.contains_key(attribute)
	}

	fn value(&self, attribute: &A, attributes: &AttributeMap<A, M>) -> Option<f32> {
		self.instances
			.get(attribute)
			.map(|attr| attr.value(attributes))
	}
	fn base_value(&self, attribute: &A) -> Option<f32> {
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
