#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "serde")]
pub(crate) trait SerdeSupport: Serialize + DeserializeOwned {}
#[cfg(feature = "serde")]
impl<T: Serialize + DeserializeOwned> SerdeSupport for T {}

#[cfg(not(feature = "serde"))]
pub trait SerdeSupport {}
#[cfg(not(feature = "serde"))]
impl<T> SerdeSupport for T {}