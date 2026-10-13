use bevy::prelude;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use tracing::info;

/// Wrapper around Option<T> to be used as an optional value within a codec
/// Used instead of Option to avoid Option enum values being included in
/// output files
pub struct Maybe<T: Serialize>(pub Option<T>);

impl<T: Serialize> Maybe<T> {
    pub fn into_inner(self) -> Option<T> {
        self.0
    }
}

impl<T: Serialize> Default for Maybe<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: Serialize> Deref for Maybe<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Serialize> Serialize for Maybe<T> {
    fn serialize<S>(&self, serializer: S) -> prelude::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            Some(value) => value.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}

impl<'de, T> Deserialize<'de> for Maybe<T>
where
    T: Deserialize<'de> + Serialize,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let result = T::deserialize(deserializer);
        let opt = Some(result?);
        Ok(Maybe(opt))
    }
}

impl<T: Serialize> From<Option<T>> for Maybe<T> {
    fn from(opt: Option<T>) -> Self {
        Maybe(opt)
    }
}

impl<T: Serialize> From<Maybe<T>> for Option<T> {
    fn from(maybe: Maybe<T>) -> Self {
        maybe.0
    }
}

impl<T: Serialize + Debug> Debug for Maybe<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Maybe").field(&self.0).finish()
    }
}

impl<T: Serialize + Clone> Clone for Maybe<T> {
    fn clone(&self) -> Self {
        Maybe(self.0.clone())
    }
}

impl<T: Serialize + Copy> Copy for Maybe<T> {}

/// Variant of Option<T> to be used when an absent value means the default should be used,
/// while still allowing for explicit declaration of None
///
/// The `Default` variant is a symbolic marker. It does not require `T: Default`
/// unless you call methods that resolve it into an actual `T`, such as [`resolve`].
#[derive(Debug, PartialEq, Eq)]
pub enum DefaultOption<T> {
    Some(T),
    Default,
    None,
}
impl<T> DefaultOption<T> {
    pub fn is_some(&self) -> bool {
        matches!(self, DefaultOption::Some(_))
    }

    pub fn is_default(&self) -> bool {
        matches!(self, DefaultOption::Default)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, DefaultOption::None)
    }

    pub fn as_ref(&self) -> DefaultOption<&T> {
        match self {
            DefaultOption::Some(value) => DefaultOption::Some(value),
            DefaultOption::Default => DefaultOption::Default,
            DefaultOption::None => DefaultOption::None,
        }
    }
    
    /// Map the contained value, if present
    ///
    /// Default values are not modified and are passed through transparently
    pub fn map<F, Out>(self, mapper: F) -> DefaultOption<Out>
    where
        F: FnOnce(T) -> Out
    {
        match self {
            DefaultOption::Some(value) => DefaultOption::Some(mapper(value)),
            DefaultOption::Default => DefaultOption::Default,
            DefaultOption::None => DefaultOption::None,
        }
    }
    
    /// Map the value if present
    ///
    /// If Default, return the default value of the mapped type (`Some(Out::default())`)
    pub fn map_or_default<F, Out>(self, mapper: F) -> Option<Out>
    where
        F: FnOnce(T) -> Out,
        Out: Default
    {
        match self {
            DefaultOption::Some(value) => Some(mapper(value)),
            DefaultOption::Default => Some(Out::default()),
            DefaultOption::None => None,
        }
    }
}
impl<T: Default> DefaultOption<T> {
    /// Resolve the default value, then return an Option with the resolved value if present
    pub fn resolve(self) -> Option<T> {
        match self {
            DefaultOption::Some(value) => Some(value),
            DefaultOption::Default => Some(T::default()),
            DefaultOption::None => None,
        }
    }

    /// Resolve the default value, then map the resolved value if present
    pub fn resolve_map<F, Out>(self, mapper: F) -> Option<Out>
    where
        F: FnOnce(T) -> Out
    {
        self.resolve().map(mapper)
    }
}
impl<T: Clone> DefaultOption<T> {
    pub fn cloned(&self) -> Self {
        match self {
            DefaultOption::Some(value) => DefaultOption::Some(value.clone()),
            DefaultOption::Default => DefaultOption::Default,
            DefaultOption::None => DefaultOption::None,
        }
    }
}
impl<T: Copy> DefaultOption<T> {
    pub fn copied(&self) -> Self {
        match self {
            DefaultOption::Some(value) => DefaultOption::Some(*value),
            DefaultOption::Default => DefaultOption::Default,
            DefaultOption::None => DefaultOption::None,
        }
    }
}

impl<T> Default for DefaultOption<T> {
    fn default() -> Self {
        info!(
            "Creating default MaybeOrDefault for {}",
            std::any::type_name::<T>()
        );
        Self::Default
    }
}

impl<T: Serialize> Serialize for DefaultOption<T> {
    fn serialize<S>(&self, serializer: S) -> prelude::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Some(value) => value.serialize(serializer),
            Self::Default => Err(serde::ser::Error::custom(
                "Serializing `DefaultOption::Default` is not supported. \
                 Ensure struct is annotated with `#[maybe_fields]` or \
                 field is annotated `#[serde(skip_serializing_if = DefaultOption::is_default)]`",
            )),
            Self::None => serializer.serialize_none(),
        }
    }
}

impl<'de, T> Deserialize<'de> for DefaultOption<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(|opt| match opt {
            Some(val) => DefaultOption::Some(val),
            None => DefaultOption::None,
        })
    }
}

impl<T> From<Option<T>> for DefaultOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => DefaultOption::Some(value),
            None => DefaultOption::None,
        }
    }
}

impl<T> From<T> for DefaultOption<T> {
    fn from(value: T) -> Self {
        DefaultOption::Some(value)
    }
}

impl<T: Clone> Clone for DefaultOption<T> {
    fn clone(&self) -> Self {
        match self {
            DefaultOption::Some(val) => DefaultOption::Some(val.clone()),
            DefaultOption::Default => DefaultOption::Default,
            DefaultOption::None => DefaultOption::None,
        }
    }
}

impl<T: Copy> Copy for DefaultOption<T> {}
