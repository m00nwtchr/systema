#![warn(clippy::pedantic)]

pub mod actor;
pub mod attribute;
pub mod system;
mod util_traits;

pub mod prelude {
	pub use crate::{
		actor::Actor,
		attribute::{
			Attribute,
			instance::AttributeInstance,
			modifier::{AttributeModifier, Operation, Value},
			supplier::{AttributeSupplier, AttributeSupplierBuilder},
		},
		system::System,
	};

	pub type AttributeMap<S> = crate::attribute::map::AttributeMap<
		<S as System>::AttributeKey,
		<S as System>::ModifierKey,
		<S as System>::AttributeValue,
		<S as System>::Operation,
	>;

	// pub type AttributeSupplier<S> = crate::attribute::supplier::AttributeSupplier<
	// 	<S as System>::AttributeKey,
	// 	<S as System>::ModifierKey,
	// 	<S as System>::AttributeValue,
	// >;
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::prelude::*;

	macro_rules! psize_of {
		($t:ty) => {
			eprintln!("{}: {} bytes", stringify!($t), std::mem::size_of::<$t>());
		};
	}

	#[test]
	fn wwww() {
		// psize_of!(attribute::map::AttributeMap<&str, &str, u8>);
		//
		// psize_of!(AttributeInstance<&str, &str, u8>);

		psize_of!(HashMap<&str, AttributeModifier<&str, u8>>);
		psize_of!(Vec<(&str, AttributeModifier<&str, u8>)>);

		psize_of!(Attribute<u8>);
	}
}
