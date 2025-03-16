use std::sync::Arc;

use once_cell::sync::Lazy;
use systema::prelude::*;

static ATTRIBUTES: Lazy<Arc<AttributeSupplier<AttributeKey, ModifierKey, u8>>> = Lazy::new(|| {
	Arc::new(
		AttributeSupplier::builder()
			.add(
				AttributeKey::MaxHealth,
				AttributeInstance::builder(Attribute::Derived)
					.modifier(
						ModifierKey::Attribute(AttributeKey::Stamina),
						AttributeModifier::new(
							Value::Attribute(AttributeKey::Stamina),
							Operation::Add,
						)
						.base(),
					)
					.modifier(
						ModifierKey::Attribute(AttributeKey::Size),
						AttributeModifier::new(
							Value::Attribute(AttributeKey::Size),
							Operation::Add,
						)
						.base(),
					),
			)
			.add(
				AttributeKey::Speed,
				AttributeInstance::builder(Attribute::Derived)
					.modifier(
						ModifierKey::Attribute(AttributeKey::Dexterity),
						AttributeModifier::new(
							Value::Attribute(AttributeKey::Dexterity),
							Operation::Add,
						)
						.base(),
					)
					.modifier(
						ModifierKey::Attribute(AttributeKey::Strength),
						AttributeModifier::new(
							Value::Attribute(AttributeKey::Strength),
							Operation::Add,
						)
						.base(),
					)
					.modifier(
						ModifierKey::Attribute(AttributeKey::Speed),
						AttributeModifier::new(Value::Value(5), Operation::Add).base(),
					),
			)
			.add(AttributeKey::Size, Attribute::Value(5))
			.add(AttributeKey::Stamina, Attribute::Value(1))
			.add(AttributeKey::Strength, Attribute::Value(1))
			.add(AttributeKey::Dexterity, Attribute::Value(1))
			.add(AttributeKey::Renown(Renown::Purity), Attribute::Value(0))
			.build(),
	)
});

struct MockActor {
	pub attributes: AttributeMap<MockSystem>,
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
							&AttributeKey::Strength,
							ModifierKey::Form(Form::Dalu),
							AttributeModifier::new(Value::Value(1), Operation::Add),
						)
						.add_modifier(
							&AttributeKey::Stamina,
							ModifierKey::Form(Form::Dalu),
							AttributeModifier::new(Value::Value(1), Operation::Add),
						)
						.add_modifier(
							&AttributeKey::Size,
							ModifierKey::Form(Form::Dalu),
							AttributeModifier::new(Value::Value(1), Operation::Add),
						);
				}
				Form::Gauru => {
					self.attributes
						.add_modifier(
							&AttributeKey::Strength,
							ModifierKey::Form(Form::Gauru),
							AttributeModifier::new(Value::Value(3), Operation::Add),
						)
						.add_modifier(
							&AttributeKey::Dexterity,
							ModifierKey::Form(Form::Gauru),
							AttributeModifier::new(Value::Value(1), Operation::Add),
						)
						.add_modifier(
							&AttributeKey::Stamina,
							ModifierKey::Form(Form::Gauru),
							AttributeModifier::new(Value::Value(2), Operation::Add),
						)
						.add_modifier(
							&AttributeKey::Size,
							ModifierKey::Form(Form::Gauru),
							AttributeModifier::new(Value::Value(2), Operation::Add),
						);
				}
				Form::Urhan => {
					self.attributes
						.add_modifier(
							&AttributeKey::Dexterity,
							ModifierKey::Form(Form::Urhan),
							AttributeModifier::new(Value::Value(2), Operation::Add),
						)
						.add_modifier(
							&AttributeKey::Stamina,
							ModifierKey::Form(Form::Urhan),
							AttributeModifier::new(Value::Value(1), Operation::Add),
						)
						.add_modifier(
							&AttributeKey::Size,
							ModifierKey::Form(Form::Urhan),
							AttributeModifier::new(Value::Value(1), Operation::Sub),
						);
				}
				Form::Urshul => {
					self.attributes
						.add_modifier(
							&AttributeKey::Strength,
							ModifierKey::Form(Form::Urshul),
							AttributeModifier::new(Value::Value(2), Operation::Add),
						)
						.add_modifier(
							&AttributeKey::Dexterity,
							ModifierKey::Form(Form::Urshul),
							AttributeModifier::new(Value::Value(2), Operation::Add),
						)
						.add_modifier(
							&AttributeKey::Stamina,
							ModifierKey::Form(Form::Urshul),
							AttributeModifier::new(Value::Value(2), Operation::Add),
						)
						.add_modifier(
							&AttributeKey::Size,
							ModifierKey::Form(Form::Urshul),
							AttributeModifier::new(Value::Value(1), Operation::Add),
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
			attributes: AttributeMap::<MockSystem>::new(ATTRIBUTES.clone()),
			form: None,
		};

		match kind {
			ActorKind::Wizard => {}
			ActorKind::Werewolf => {
				_self.form = Some(Form::Hishu);
				_self.set_form(Form::Hishu);

				// #[cfg(not(feature = "serde"))]
				// _self.attributes.add_modifier(
				// 	AttributeKey::MaxHealth,
				// 	ModifierKey::WarriorsHide,
				// 	AttributeModifier::new(
				// 		Value::Attribute(AttributeKey::Renown(Renown::Purity)),
				// 		Operation::Fn(Box::new(
				// 			|v, renown| if renown >= 2 { v + renown } else { v },
				// 		)),
				// 	),
				// );
			}
		}

		_self
	}

	fn attributes(&self) -> &AttributeMap<MockSystem> {
		&self.attributes
	}

	fn attributes_mut(&mut self) -> &mut AttributeMap<MockSystem> {
		&mut self.attributes
	}
}

struct MockSystem;

impl System for MockSystem {
	type AttributeKey = AttributeKey;
	type ModifierKey = ModifierKey;
	type AttributeValue = u8;
	type Operation = Operation;

	type Actor = MockActor;
}

#[derive(PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActorKind {
	Wizard,
	Werewolf,
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AttributeKey {
	MaxHealth,
	Speed,

	Size,

	Stamina,
	Strength,
	Dexterity,

	Renown(Renown), // Form,
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Form {
	Hishu,
	Dalu,
	Gauru,
	Urhan,
	Urshul,
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Renown {
	Purity,
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModifierKey {
	Form(Form),
	WarriorsHide,

	Attribute(AttributeKey),

	Test,
}

#[test]
fn it_works() {
	let mut actor = MockActor::new(ActorKind::Werewolf);
	assert_eq!(Some(6), actor.attributes.value(&AttributeKey::MaxHealth));

	actor.set_form(Form::Gauru);
	assert_eq!(Some(10), actor.attributes.value(&AttributeKey::MaxHealth));

	actor.attributes.add_modifier(
		&AttributeKey::MaxHealth,
		ModifierKey::Test,
		AttributeModifier::new(Value::Value(1), Operation::Add),
	);
	assert_eq!(Some(11), actor.attributes.value(&AttributeKey::MaxHealth));
	actor
		.attributes
		.remove_modifier(&AttributeKey::MaxHealth, &ModifierKey::Test);

	actor
		.attributes
		.set_raw_value(&AttributeKey::Renown(Renown::Purity), 1);
	assert_eq!(Some(10), actor.attributes.value(&AttributeKey::MaxHealth));
	assert_eq!(
		Some(1),
		actor
			.attributes
			.base_value(&AttributeKey::Renown(Renown::Purity))
	);

	actor
		.attributes
		.set_raw_value(&AttributeKey::Renown(Renown::Purity), 2);
	// assert_eq!(Some(12), actor.attributes.value(&AttributeKey::MaxHealth));
	assert_eq!(
		Some(10),
		actor.attributes.base_value(&AttributeKey::MaxHealth)
	);

	actor.set_form(Form::Hishu);
	// assert_eq!(Some(8), actor.attributes.value(&AttributeKey::MaxHealth));

	assert_eq!(actor.attributes.value(&AttributeKey::Dexterity), Some(1));
	assert_eq!(actor.attributes.value(&AttributeKey::Strength), Some(1));
	assert_eq!(actor.attributes.value(&AttributeKey::Speed), Some(7))
}
