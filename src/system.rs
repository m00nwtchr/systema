use crate::{
	actor::Actor,
	util_traits::{Key, Number},
};

pub trait System {
	type AttributeKey: Key;
	type ModifierKey: Key;
	type AttributeValue: Number;

	type Actor: Actor<System = Self>;
}
