use itertools::Itertools;
use std::cell::Cell;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use crate::attribute::map::AttributeMap;
use crate::serde_support::SerdeSupport;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod map;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub enum Attribute {
	Value(f32),
	Ranged(f32, f32, f32),
}

impl Attribute {
	pub fn default_value(&self) -> f32 {
		match self {
			Attribute::Value(d) => *d,
			Attribute::Ranged(d, _, _) => *d,
		}
	}

	pub fn sanitize_value(&self, value: f32) -> f32 {
		match self {
			Self::Value(_) => value,
			Self::Ranged(_, min, max) => {
				if value.is_nan() {
					*min
				} else {
					value.clamp(*min, *max)
				}
			}
		}
	}
}

#[cfg_attr(feature = "serde", derive(Serialize))]
// #[derive(Clone)]
pub struct AttributeInstance<A, M>
where
	A: Clone + PartialEq + Eq + Hash + SerdeSupport,
	M: Clone + SerdeSupport,
{
	#[cfg_attr(feature = "serde", serde(skip))]
	attribute: Arc<Attribute>,
	#[cfg_attr(feature = "serde", serde(skip))]
	modifiers: HashMap<M, AttributeModifier<A>>,

	base_value: f32,
	#[cfg_attr(feature = "serde", serde(skip))]
	cached_value: Mutex<Option<f32>>,
	// on_dirty: Box<dyn FnMut()>,
}
impl<A,M> Clone for AttributeInstance<A, M>
where
	A: Clone + PartialEq + Eq + Hash + SerdeSupport,
	M: Clone + SerdeSupport,
{
	fn clone(&self) -> Self {
		Self {
			attribute: self.attribute.clone(),
			modifiers: self.modifiers.clone(),
			base_value: self.base_value,
			cached_value: Mutex::new(*self.cached_value.lock().unwrap())
		}
	}
}

impl<A, M> AttributeInstance<A, M>
where
	M: Clone + PartialEq + Eq + Hash + SerdeSupport,
	A: Clone + PartialEq + Eq + Hash + SerdeSupport,
{
	pub fn new(
		attribute: Arc<Attribute>,
		// on_dirty: Box<dyn Fn(&A)>
	) -> Self {
		let base_value = attribute.default_value();
		Self {
			attribute,
			modifiers: HashMap::default(),
			base_value,
			cached_value: Mutex::new(None),
			// on_dirty
		}
	}

	pub fn base_value(&self) -> f32 {
		self.base_value
	}

	fn set_dirty(&mut self) {
		self.cached_value.lock().unwrap().take();
	}

	pub fn set_base_value(&mut self, value: f32) {
		if value != self.base_value {
			self.base_value = value;
			self.set_dirty();
		}
	}

	fn compute_value(&self, attributes: &AttributeMap<A, M>) -> f32 {
		let mut value = self.base_value;

		for (id, modifier) in &self.modifiers {
			let mod_val = match &modifier.value {
				Value::Value(val) => *val,
				Value::Attribute(attr) => attributes.value(attr).unwrap_or_default(),
			};

			value = match modifier.op {
				Operation::Add => value + mod_val,
			};
		}

		self.attribute.sanitize_value(value)
	}

	pub fn value(&self, attributes: &AttributeMap<A, M>) -> f32 {
		let mut cached_value = self.cached_value.lock().unwrap();
		match *cached_value {
			None => {
				let val = self.compute_value(attributes);
				cached_value.replace(val);
				val
			}
			Some(val) => val,
		}
	}

	pub fn has_modifier(&self, modifier: &M) -> bool {
		self.modifiers.contains_key(modifier)
	}
	pub fn modifier(&self, modifier: &M) -> Option<&AttributeModifier<A>> {
		self.modifiers.get(modifier)
	}

	pub fn add_modifier(&mut self, id: M, modifier: AttributeModifier<A>) {
		self.modifiers.insert(id, modifier);
		self.set_dirty();
	}

	pub fn dependencies(&self) -> Box<[&A]> {
		self.modifiers
			.iter()
			.filter_map(|(k, v)| match &v.value {
				Value::Value(_) => None,
				Value::Attribute(a) => Some(a),
			})
			.unique()
			.collect()
	}
}

impl<A, M> Default for AttributeInstance<A, M>
where
	M: Clone + PartialEq + Eq + Hash + SerdeSupport,
	A: Clone + PartialEq + Eq + Hash + SerdeSupport,
{
	fn default() -> Self {
		AttributeInstance::new(Arc::new(Attribute::Value(0.0)))
	}
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub enum Operation {
	Add,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub enum Value<A> {
	Value(f32),
	Attribute(A),
}

impl<A> From<f32> for Value<A> {
	fn from(value: f32) -> Self {
		Self::Value(value)
	}
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct AttributeModifier<A> {
	value: Value<A>,
	op: Operation,
}

impl<A> AttributeModifier<A> {
	pub fn new<I: Into<Value<A>>>(value: I, op: Operation) -> Self {
		Self {
			value: value.into(),
			op,
		}
	}
}
