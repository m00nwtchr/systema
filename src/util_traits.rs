use std::hash::Hash;

#[cfg(feature = "serde")]
use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "serde")]
pub(crate) trait SerdeSupport: Serialize + DeserializeOwned {}
#[cfg(feature = "serde")]
impl<T: Serialize + DeserializeOwned> SerdeSupport for T {}

#[cfg(not(feature = "serde"))]
pub trait SerdeSupport {}
#[cfg(not(feature = "serde"))]
impl<T> SerdeSupport for T {}

pub trait Number:
	SerdeSupport
	+ Copy
	+ PartialEq
	+ PartialOrd
	+ Default
	+ std::ops::Add<Self, Output = Self>
	+ 'static
{
}

impl<T> Number for T where
	T: SerdeSupport
		+ Copy
		+ PartialEq
		+ PartialOrd
		+ Default
		+ std::ops::Add<Self, Output = Self>
		+ 'static
{
}

pub trait Key: Clone + PartialEq + Eq + Hash + SerdeSupport {}
impl<T> Key for T where T: Clone + PartialEq + Eq + Hash + SerdeSupport {}
