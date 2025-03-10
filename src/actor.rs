use crate::{prelude::AttributeMap, system::System};

pub trait Actor {
	type System: System;
	type Kind;

	fn new(kind: Self::Kind) -> Self;

	fn attributes(&self) -> &AttributeMap<Self::System>;
	fn attributes_mut(&mut self) -> &mut AttributeMap<Self::System>;
}
