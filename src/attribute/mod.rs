use std::{
	cmp::Ordering,
	collections::HashMap,
	sync::{Arc, Mutex},
};

use itertools::Itertools;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
	attribute::map::AttributeMap,
	util_traits::{Key, Number},
};

pub mod map;
mod supplier;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub enum Attribute<V = f32>
where
	V: Number + 'static,
{
	Value(V),
	Ranged(V, V, V),
}

pub fn clamp<T: PartialOrd + Copy>(value: T, min: T, max: T) -> T {
	// value is NaN or less than min
	match value.partial_cmp(&min) {
		None | Some(Ordering::Less) => min,
		_ => {
			if value > max {
				max
			} else {
				value
			}
		}
	}
}

impl<V> Attribute<V>
where
	V: Number + 'static,
{
	pub fn default_value(&self) -> V {
		match self {
			Attribute::Value(d) => *d,
			Attribute::Ranged(d, _, _) => *d,
		}
	}

	pub fn sanitize_value(&self, value: V) -> V {
		match self {
			Self::Value(_) => value,
			Self::Ranged(_, min, max) => clamp(value, *min, *max),
		}
	}
}

#[cfg_attr(feature = "serde", derive(Serialize))]
// #[derive(Clone)]
pub struct AttributeInstance<A, M, V = f32>
where
	A: Key,
	M: Key,
	V: Number + 'static,
{
	#[cfg_attr(feature = "serde", serde(skip))]
	attribute: Arc<Attribute<V>>,
	#[cfg_attr(feature = "serde", serde(skip))]
	modifiers: HashMap<M, AttributeModifier<A, V>>,

	base_value: V,
	#[cfg_attr(feature = "serde", serde(skip))]
	cached_value: Mutex<Option<V>>,
	// on_dirty: Box<dyn FnMut()>,
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
			base_value: self.base_value,
			cached_value: Mutex::new(*self.cached_value.lock().unwrap()),
		}
	}
}

impl<A, M, V> AttributeInstance<A, M, V>
where
	M: Key,
	A: Key,
	V: Number + 'static,
{
	pub fn new(
		attribute: Arc<Attribute<V>>,
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

	pub fn base_value(&self) -> V {
		self.base_value
	}

	fn mark_dirty(&mut self) {
		self.cached_value.lock().unwrap().take();
	}

	pub fn set_base_value(&mut self, value: V) {
		if value != self.base_value {
			self.base_value = value;
			self.mark_dirty();
		}
	}

	fn compute_value(&self, attributes: &AttributeMap<A, M, V>) -> V {
		let mut value = self.base_value;

		for modifier in self.modifiers.values() {
			let mod_val = match &modifier.value {
				Value::Value(val) => *val,
				Value::Attribute(attr) => attributes.value(attr).unwrap_or_default(),
			};

			value = match &modifier.op {
				Operation::Add => value + mod_val,
				Operation::Fn(f) => f(value, mod_val),
			};
		}

		self.attribute.sanitize_value(value)
	}

	pub fn value(&self, attributes: &AttributeMap<A, M, V>) -> V {
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
	pub fn modifier(&self, modifier: &M) -> Option<&AttributeModifier<A, V>> {
		self.modifiers.get(modifier)
	}

	pub fn add_modifier(&mut self, id: M, modifier: AttributeModifier<A, V>) {
		self.modifiers.insert(id, modifier);
		self.mark_dirty();
	}

	pub fn remove_modifier(&mut self, id: &M) -> Option<AttributeModifier<A, V>> {
		let modifier = self.modifiers.remove(id);
		if modifier.is_some() {
			self.mark_dirty();
		}
		modifier
	}

	// pub fn dependencies(&self) -> Box<[&A]> {
	// 	self.modifiers
	// 		.iter()
	// 		.filter_map(|(_, v)| match &v.value {
	// 			Value::Value(_) => None,
	// 			Value::Attribute(a) => Some(a),
	// 		})
	// 		.unique()
	// 		.collect()
	// }

	pub fn depends_on(&self, attr: &A) -> bool {
		self.modifiers
			.values()
			.any(|modifier| modifier.value.is_attribute(attr))
	}
}

impl<A, M> Default for AttributeInstance<A, M>
where
	M: Key,
	A: Key,
{
	fn default() -> Self {
		AttributeInstance::new(Arc::new(Attribute::Value(0.0)))
	}
}

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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub enum Operation<V: 'static> {
	Add,
	Fn(Box<dyn CloneableFn<V>>),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq)]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct AttributeModifier<A, V: 'static> {
	value: Value<A, V>,
	op: Operation<V>,
}

impl<A, V> AttributeModifier<A, V> {
	pub fn new<I: Into<Value<A, V>>>(value: I, op: Operation<V>) -> Self {
		Self {
			value: value.into(),
			op,
		}
	}
}
