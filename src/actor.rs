use crate::attribute::map::AttributeMap;
use std::hash::Hash;

use crate::serde_support::SerdeSupport;
#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg_attr(feature = "serde", derive(Serialize))]
pub trait Actor
{
	type Kind;
	type AttributeKey: Clone + PartialEq + Eq + Hash + SerdeSupport;
	type ModifierKey: Clone + PartialEq + Eq + Hash + SerdeSupport;
	// pub attributes: AttributeMap<A, M>,

	fn new(kind: Self::Kind) -> Self;

	fn attributes(&self) -> &AttributeMap<Self::AttributeKey, Self::ModifierKey>;
	fn attributes_mut(&mut self) -> &mut AttributeMap<Self::AttributeKey, Self::ModifierKey>;
}

