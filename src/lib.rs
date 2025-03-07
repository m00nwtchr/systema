mod actor;
mod attribute;
mod holder;
mod system;
mod util_traits;

pub mod prelude {
	pub use crate::{
		actor::Actor,
		attribute::{
			Attribute, AttributeInstance, AttributeModifier, Operation, Value,
			supplier::{AttributeSupplier, AttributeSupplierBuilder},
		},
		system::System,
	};

	pub type AttributeMap<S> = crate::attribute::map::AttributeMap<
		<S as System>::AttributeKey,
		<S as System>::ModifierKey,
		<S as System>::AttributeValue,
	>;
}

#[cfg(test)]
mod tests {
	use once_cell::sync::Lazy;
	#[cfg(feature = "serde")]
	use serde::{Deserialize, Serialize};

	use super::prelude::*;
	use crate::attribute::map::AttributeMap;

	static ATTRIBUTES: Lazy<AttributeSupplier<AttributeKey, ModifierKey>> = Lazy::new(|| {
		AttributeSupplier::builder()
			.create(AttributeKey::MaxHealth, Attribute::Ranged(0.0, 1.0, 1024.0))
			.modifier(
				ModifierKey::Attribute(AttributeKey::Stamina),
				AttributeModifier::new(Value::Attribute(AttributeKey::Stamina), Operation::Add),
			)
			.modifier(
				ModifierKey::Attribute(AttributeKey::Size),
				AttributeModifier::new(Value::Attribute(AttributeKey::Size), Operation::Add),
			)
			.insert()
			.create(AttributeKey::Speed, Attribute::Value(5.0))
			.modifier(
				ModifierKey::Attribute(AttributeKey::Dexterity),
				AttributeModifier::new(Value::Attribute(AttributeKey::Dexterity), Operation::Add),
			)
			.modifier(
				ModifierKey::Attribute(AttributeKey::Strength),
				AttributeModifier::new(Value::Attribute(AttributeKey::Strength), Operation::Add),
			)
			.insert()
			.add(AttributeKey::Size, Attribute::Value(5.0))
			.add(AttributeKey::Stamina, Attribute::Value(1.0))
			.add(AttributeKey::Strength, Attribute::Value(1.0))
			.add(AttributeKey::Dexterity, Attribute::Value(1.0))
			.build()
	});

	struct MockActor {
		pub attributes: AttributeMap<AttributeKey, ModifierKey>,
		pub form: Option<Form>,
	}
	impl MockActor {
		pub fn set_form(&mut self, form: Form) {
			if let Some(form_mut) = self.form.as_mut() {
				let old_form = std::mem::replace(form_mut, form);
				self.attributes
					.remove_modifiers(&ModifierKey::Form(old_form));

				match form_mut {
					Form::Hishu => {}
					Form::Dalu => {
						self.attributes
							.add_modifier(
								AttributeKey::Strength,
								ModifierKey::Form(Form::Dalu),
								AttributeModifier::new(Value::Value(1.0), Operation::Add),
							)
							.add_modifier(
								AttributeKey::Stamina,
								ModifierKey::Form(Form::Dalu),
								AttributeModifier::new(Value::Value(1.0), Operation::Add),
							)
							.add_modifier(
								AttributeKey::Size,
								ModifierKey::Form(Form::Dalu),
								AttributeModifier::new(Value::Value(1.0), Operation::Add),
							);
					}
					Form::Gauru => {
						self.attributes
							.add_modifier(
								AttributeKey::Strength,
								ModifierKey::Form(Form::Gauru),
								AttributeModifier::new(Value::Value(3.0), Operation::Add),
							)
							.add_modifier(
								AttributeKey::Dexterity,
								ModifierKey::Form(Form::Gauru),
								AttributeModifier::new(Value::Value(1.0), Operation::Add),
							)
							.add_modifier(
								AttributeKey::Stamina,
								ModifierKey::Form(Form::Gauru),
								AttributeModifier::new(Value::Value(2.0), Operation::Add),
							)
							.add_modifier(
								AttributeKey::Size,
								ModifierKey::Form(Form::Gauru),
								AttributeModifier::new(Value::Value(2.0), Operation::Add),
							);
					}
					Form::Urhan => {
						self.attributes
							.add_modifier(
								AttributeKey::Dexterity,
								ModifierKey::Form(Form::Urhan),
								AttributeModifier::new(Value::Value(2.0), Operation::Add),
							)
							.add_modifier(
								AttributeKey::Stamina,
								ModifierKey::Form(Form::Urhan),
								AttributeModifier::new(Value::Value(1.0), Operation::Add),
							)
							.add_modifier(
								AttributeKey::Size,
								ModifierKey::Form(Form::Urhan),
								AttributeModifier::new(Value::Value(-1.0), Operation::Add),
							);
					}
					Form::Urshul => {
						self.attributes
							.add_modifier(
								AttributeKey::Strength,
								ModifierKey::Form(Form::Urshul),
								AttributeModifier::new(Value::Value(2.0), Operation::Add),
							)
							.add_modifier(
								AttributeKey::Dexterity,
								ModifierKey::Form(Form::Urshul),
								AttributeModifier::new(Value::Value(2.0), Operation::Add),
							)
							.add_modifier(
								AttributeKey::Stamina,
								ModifierKey::Form(Form::Urshul),
								AttributeModifier::new(Value::Value(2.0), Operation::Add),
							)
							.add_modifier(
								AttributeKey::Size,
								ModifierKey::Form(Form::Urshul),
								AttributeModifier::new(Value::Value(1.0), Operation::Add),
							);
					}
				}
			}
		}
	}
	impl Actor for MockActor {
		type System = MockSystem;
		type Kind = ActorKind;

		fn new(kind: ActorKind) -> Self {
			let mut _self = Self {
				attributes: AttributeMap::new(&ATTRIBUTES),
				form: if matches!(kind, ActorKind::Werewolf) {
					Some(Form::Hishu)
				} else {
					None
				},
			};

			match kind {
				ActorKind::Wizard => {}
				ActorKind::Werewolf => _self.set_form(Form::Hishu),
			}

			_self
		}

		fn attributes(&self) -> &AttributeMap<AttributeKey, ModifierKey> {
			&self.attributes
		}

		fn attributes_mut(&mut self) -> &mut AttributeMap<AttributeKey, ModifierKey> {
			&mut self.attributes
		}
	}

	struct MockSystem;

	impl System for MockSystem {
		type AttributeKey = AttributeKey;
		type ModifierKey = ModifierKey;
		type AttributeValue = f32;

		type Actor = MockActor;
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

		Size,

		Stamina,
		Strength,
		Dexterity,
		// Form,
	}

	#[derive(PartialEq, Eq, Hash, Clone)]
	#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
	pub enum Form {
		Hishu,
		Dalu,
		Gauru,
		Urhan,
		Urshul,
	}

	#[derive(PartialEq, Eq, Hash, Clone)]
	#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
	pub enum ModifierKey {
		Form(Form),

		Attribute(AttributeKey),
	}

	#[test]
	fn wwww() {
		println!("{}", std::mem::size_of::<MockActor>());
		println!("{}", std::mem::size_of::<Box<[&AttributeKey]>>());
		println!("{}", std::mem::size_of::<Vec<&AttributeKey>>());
	}

	#[test]
	fn it_works() {
		let mut actor = MockActor::new(ActorKind::Werewolf);
		assert_eq!(Some(6.0), actor.attributes.value(&AttributeKey::MaxHealth));

		actor.set_form(Form::Gauru);
		assert_eq!(Some(10.0), actor.attributes.value(&AttributeKey::MaxHealth));

		actor.set_form(Form::Hishu);
		assert_eq!(actor.attributes.value(&AttributeKey::Dexterity), Some(1.0));
		assert_eq!(actor.attributes.value(&AttributeKey::Strength), Some(1.0));
		assert_eq!(actor.attributes.value(&AttributeKey::Speed), Some(7.0))
	}
}
