use crate::{
	actor::Actor,
	attribute::modifier::Op,
	util_traits::{Key, Number},
};

pub trait System {
	type AttributeKey: Key;
	type ModifierKey: Key;
	type AttributeValue: Number;
	type Operation: Op<Self::AttributeValue>;

	type Actor: Actor<System = Self>;
}
