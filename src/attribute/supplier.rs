use std::{collections::HashMap, hash::Hash};

use crate::{
	attribute::{instance::AttributeInstance, map::AttributeMap, modifier::Op},
	prelude::Operation,
	util_traits::{Key, Number},
};

#[must_use]
pub struct AttributeSupplierBuilder<A, M, V, O = Operation>
where
	A: Key + Hash + 'static,
	M: Key + 'static,
	V: Number + 'static,
	O: Op<V>,
{
	instances: HashMap<A, AttributeInstance<A, M, V, O>>,
}

impl<A, M, V, O> AttributeSupplierBuilder<A, M, V, O>
where
	A: Key + Hash + 'static,
	M: Key + 'static,
	V: Number + 'static,
	O: Op<V>,
{
	#[must_use]
	pub fn build(self) -> AttributeSupplier<A, M, V, O> {
		AttributeSupplier {
			instances: self.instances,
		}
	}

	pub fn add<I: Into<A>, AI: Into<AttributeInstance<A, M, V, O>>>(
		mut self,
		id: I,
		attribute: AI,
	) -> Self {
		self.instances.insert(id.into(), attribute.into());
		self
	}
}

pub struct AttributeSupplier<A, M, V = f32, O = Operation>
where
	A: Key + Hash,
	M: Key,
	V: Number + 'static,
	O: Op<V>,
{
	instances: HashMap<A, AttributeInstance<A, M, V, O>>,
}

impl<A, M, V, O> AttributeSupplier<A, M, V, O>
where
	A: Key + Hash,
	M: Key,
	V: Number + 'static,
	O: Op<V>,
{
	pub fn builder() -> AttributeSupplierBuilder<A, M, V, O> {
		AttributeSupplierBuilder {
			instances: HashMap::new(),
		}
	}

	pub fn create_instance(&self, attribute: &A) -> Option<AttributeInstance<A, M, V, O>> {
		self.instances.get(attribute).cloned()
	}

	// pub(crate) fn has_attribute(&self, attribute: &A) -> bool {
	// 	self.instances.contains_key(attribute)
	// }

	pub(crate) fn value(&self, attribute: &A, attributes: &AttributeMap<A, M, V, O>) -> Option<V> {
		self.instances
			.get(attribute)
			.map(|attr| attr.compute_value(attributes, false))
	}

	pub(crate) fn base_value(
		&self,
		attribute: &A,
		attributes: &AttributeMap<A, M, V, O>,
	) -> Option<V> {
		self.instances
			.get(attribute)
			.map(|attr| attr.compute_value(attributes, true))
	}
	// pub(crate) fn raw_value(&self, attribute: &A) -> Option<V> {
	// 	self.instances.get(attribute).map(|attr| attr.raw_value())
	// }
}

impl<A, M, V, O> Default for AttributeSupplier<A, M, V, O>
where
	A: Key + Hash,
	M: Key,
	V: Number + 'static,
	O: Op<V>,
{
	fn default() -> Self {
		Self {
			instances: HashMap::new(),
		}
	}
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
	use std::sync::Arc;

	use super::*;
	use crate::prelude::{Attribute, AttributeModifier, Operation};

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

	fn mock_supplier() -> Arc<MockSupplier> {
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
	}

	#[test]
	fn test_supplier_builder() {
		let supplier = mock_supplier();

		assert_eq!(supplier.instances.len(), 2);
		assert!(supplier.instances.contains_key(&TestAttribute::Strength));
		assert!(supplier.instances.contains_key(&TestAttribute::Agility));
	}

	#[test]
	fn test_create_instance() {
		let supplier = Arc::new(
			MockSupplier::builder()
				.add(TestAttribute::Strength, Attribute::Value(1.0))
				.build(),
		);

		let instance = supplier.create_instance(&TestAttribute::Strength);
		assert!(instance.is_some());
		assert_eq!(
			instance
				.unwrap()
				.compute_value(&AttributeMap::default(), false),
			1.0
		);

		let instance_none = supplier.create_instance(&TestAttribute::Agility);
		assert!(instance_none.is_none());
	}

	#[test]
	fn test_value_computation() {
		let supplier = mock_supplier();

		let value = supplier.value(&TestAttribute::Strength, &AttributeMap::default());
		assert_eq!(value, Some(2.0));

		let base_value = supplier.base_value(&TestAttribute::Strength, &AttributeMap::default());
		assert_eq!(base_value, Some(1.0));
	}

	#[test]
	fn test_value_not_found() {
		let supplier = MockSupplier::default();

		// Testing for an attribute not in the supplier
		let non_existent_value = supplier.value(&TestAttribute::Agility, &AttributeMap::default());
		assert_eq!(non_existent_value, None);
	}

	#[test]
	fn test_default_supplier() {
		let supplier: MockSupplier = AttributeSupplier::default();
		assert!(supplier.instances.is_empty());
	}
}
