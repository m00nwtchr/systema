use std::{
	hash::Hash,
	ops::{Add, Sub},
};

// #[cfg(feature = "serde")]
// use serde::{Serialize, de::DeserializeOwned};
//
// #[cfg(feature = "serde")]
// pub(crate) trait SerdeSupport: Serialize + DeserializeOwned {}
// #[cfg(feature = "serde")]
// impl<T: Serialize + DeserializeOwned> SerdeSupport for T {}
//
// #[cfg(not(feature = "serde"))]
// pub trait SerdeSupport {}
// #[cfg(not(feature = "serde"))]
// impl<T> SerdeSupport for T {}

pub trait Number:
	Copy
	+ PartialEq
	+ PartialOrd
	+ Default
	+ Add<Self, Output = Self>
	+ Sub<Self, Output = Self>
	+ 'static
{
}

impl<T> Number for T where
	T: Copy
		+ PartialEq
		+ PartialOrd
		+ Default
		+ Add<Self, Output = Self>
		+ Sub<Self, Output = Self>
		+ 'static
{
}

pub trait Key: Clone + PartialEq + Eq {}
impl<T> Key for T where T: Clone + PartialEq + Eq {}
