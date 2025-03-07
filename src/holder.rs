// pub struct KeyHolder<R, T> {
// 	key: R,
// 	_phantom: PhantomData<T>,
// }
//
// pub struct Holder<R, T> {
// 	key: R,
// 	value: Arc<T>,
// }
//
// impl<R, T> KeyHolder<R, T> {
// 	pub fn key(&self) -> &R {
// 		&self.key
// 	}
//
// 	pub fn bind<I: Into<Arc<T>>>(self, value: I) -> Holder<R, T> {
// 		Holder {
// 			key: self.key,
// 			value: value.into(),
// 		}
// 	}
// }
//
// impl<R, T> Holder<R, T> {
// 	pub fn key(&self) -> &R {
// 		&self.key
// 	}
//
// 	pub fn value(&self) -> &T {
// 		&self.value
// 	}
// }
//
// impl<R, T> From<R> for KeyHolder<R, T> {
// 	fn from(key: R) -> Self {
// 		Self {
// 			key,
// 			_phantom: PhantomData,
// 		}
// 	}
// }
//
// impl<R, T> From<(R, T)> for Holder<R, T> {
// 	fn from(value: (R, T)) -> Self {
// 		Self {
// 			key: value.0,
// 			value: value.1.into(),
// 		}
// 	}
// }

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct AttributeResource(Box<str>);

impl From<String> for AttributeResource {
	fn from(value: String) -> Self {
		Self(value.into_boxed_str())
	}
}

impl From<&str> for AttributeResource {
	fn from(value: &str) -> Self {
		Self(value.into())
	}
}
