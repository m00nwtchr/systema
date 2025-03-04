use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use crate::serde_support::SerdeSupport;

mod actor;
mod attribute;
mod holder;
mod serde_support;

use crate::actor::Actor;
use crate::attribute::map::AttributeSupplier;
use crate::holder::AttributeResource;
use attribute::map::AttributeMap;

pub mod prelude {
	pub use super::System;
	pub use crate::actor::Actor;
	pub use crate::attribute::map::{AttributeSupplier, AttributeSupplierBuilder};
	pub use crate::attribute::{Attribute, AttributeInstance, AttributeModifier, Operation, Value};
}

pub trait System

{
	// actor_attributes: Arc<HashMap<A, Attribute>>,
	// attribute_suppliers: HashMap<AT, Arc<AttributeSupplier<A, M>>>,
	type Actor: Actor;

   fn new_actor(&self, kind: <<Self as System>::Actor as Actor>::Kind) -> Self::Actor;
}
//
// impl<AT, A, M> System<AT, A, M>
// where
// 	AT: PartialEq + Eq + Hash,
// 	A: Clone + PartialEq + Eq + Hash + SerdeSupport,
// 	M: Clone + PartialEq + Eq + Hash + SerdeSupport,
// {
// 	pub fn new(suppliers: HashMap<AT, Arc<AttributeSupplier<A, M>>>) -> Self {
// 		Self {
// 			attribute_suppliers: suppliers,
// 		}
// 	}
//
// 	pub fn new_actor(&self, kind: &AT) -> Actor<A, M> {
// 		Actor {
// 			attributes: AttributeMap::new(self.attribute_suppliers.get(kind).expect(" ").clone()),
// 		}
// 	}
// }

#[cfg(test)]
mod tests {
	use super::prelude::*;
	use std::collections::HashMap;
	use std::sync::Arc;
	use once_cell::sync::Lazy;
	#[cfg(feature = "serde")]
	use serde::{Deserialize, Serialize};
	use crate::attribute::map::AttributeMap;
	use crate::holder::AttributeResource;

	static ATTRIBUTES: Lazy<AttributeSupplier<AttributeKey, ModifierKey>> = Lazy::new(|| {
	AttributeSupplier::builder()
			.add(
				AttributeKey::MaxHealth,
				Attribute::Ranged(20.0, 0.0, 1024.0),
			)
			.add(AttributeKey::Speed, Attribute::Value(5.0))
			.add(AttributeKey::Strength, Attribute::Value(2.0))
			.add(AttributeKey::Dexterity, Attribute::Value(2.0))
			.build()
	});

	struct MockActor {
		pub attributes: AttributeMap<AttributeKey, ModifierKey>,
	}
	impl Actor for MockActor {
		type Kind = ActorKind;
		type AttributeKey = AttributeKey;
		type ModifierKey = ModifierKey;

		fn new(kind: ActorKind) -> Self {
			Self {
				attributes: AttributeMap::new(&ATTRIBUTES)
			}
		}
		
		fn attributes(&self) -> &AttributeMap<AttributeKey, ModifierKey> {
			&self.attributes
		}

		fn attributes_mut(&mut self) -> &mut AttributeMap<AttributeKey, ModifierKey> {
			&mut self.attributes
		}
	}

	struct MockSystem {

	}

	impl MockSystem {
		fn new() -> Self {
			Self {}
		}
	}
	impl System for MockSystem {
		type Actor = MockActor;

		fn new_actor(&self, kind: ActorKind) -> Self::Actor {
			Self::Actor::new(kind)
		}
	}

	#[derive(PartialEq, Eq, Hash)]
	#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
	pub enum ActorKind {
		Wizard,
		Werewolf,
	}

	#[derive(PartialEq, Eq, Hash, Clone)]
	#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
	pub enum AttributeKey {
		MaxHealth,
		Speed,

		Strength,
		Dexterity,
	}

	#[derive(PartialEq, Eq, Hash, Clone)]
	#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
	pub enum ModifierKey {
		GauruForm,

		Attribute(AttributeKey),
	}

	#[test]
	fn wwww() {
		println!("{}", std::mem::size_of::<Box<[&AttributeKey]>>());
		println!("{}", std::mem::size_of::<Vec<&AttributeKey>>());
	}

	#[test]
	fn it_works() {
		// HashMap::from([
		// 	(
		// 		AttributeKey::MaxHealth,
		// 		Attribute::Ranged(20.0, 0.0, 1024.0),
		// 	),
		// 	(AttributeKey::Speed, Attribute::Value(5.0)),
		// ])

		let system = MockSystem::new();

		// let system: MockSystem = System::new(HashMap::from([(
		// 	ActorKind::Werewolf,
		// 	Arc::new(
		// 		AttributeSupplier::builder()
		// 			.add(
		// 				AttributeKey::MaxHealth,
		// 				Attribute::Ranged(20.0, 0.0, 1024.0),
		// 			)
		// 			.add(AttributeKey::Speed, Attribute::Value(5.0))
		// 			.add(AttributeKey::Strength, Attribute::Value(2.0))
		// 			.add(AttributeKey::Dexterity, Attribute::Value(2.0))
		// 			.build(),
		// 	),
		// )]));

		let mut actor = system.new_actor(ActorKind::Werewolf);
		assert_eq!(Some(20.0), actor.attributes.value(&AttributeKey::MaxHealth));

		actor.attributes.add_modifier(
			AttributeKey::MaxHealth,
			ModifierKey::GauruForm,
			AttributeModifier::new(2.0, Operation::Add),
		);
		assert_eq!(Some(22.0), actor.attributes.value(&AttributeKey::MaxHealth));

		actor
			.attributes
			.set_base_value(AttributeKey::MaxHealth, 5.0);
		assert_eq!(Some(7.0), actor.attributes.value(&AttributeKey::MaxHealth));

		// let speed = actor.attributes.attribute_mut(AttributeKey::Speed).unwrap();
		actor.attributes.add_modifier(
			AttributeKey::Speed,
			ModifierKey::Attribute(AttributeKey::Dexterity),
			AttributeModifier::new(Value::Attribute(AttributeKey::Dexterity), Operation::Add),
		);
		actor.attributes.add_modifier(
			AttributeKey::Speed,
			ModifierKey::Attribute(AttributeKey::Strength),
			AttributeModifier::new(Value::Attribute(AttributeKey::Strength), Operation::Add),
		);

		assert_eq!(actor.attributes.value(&AttributeKey::Speed), Some(9.0))
	}
}
