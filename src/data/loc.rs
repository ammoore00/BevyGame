use crate::data::prelude::ResourceRegistry;
use crate::data::resource::ResourceKind;
use bevy::asset::{AssetPath, Handle};
use bevy::prelude;
use bevy::prelude::Reflect;
use getset::Getters;
use regex::Regex;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::LazyLock;

trait ResourceLoc: FromStr {
    fn new(namespace: Namespace, id: ResourceId) -> Self;

    fn from_str_impl(s: &str) -> prelude::Result<Self, ResourceLocationParseError> {
        let mut split = s.split(':');
        let count = split.clone().count();

        match count {
            1 => {
                let namespace = Namespace::default();
                let id = ResourceId::from_str(s)?;
                Ok(Self::new(namespace, id))
            }
            2 => {
                let namespace = Namespace::from_str(split.next().unwrap())?;
                let id = ResourceId::from_str(split.next().unwrap())?;
                Ok(Self::new(namespace, id))
            }
            0 => Err(ResourceLocationParseError::Empty),
            _ => Err(ResourceLocationParseError::MultipleDividers(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Getters, Reflect)]
pub struct ResourceLocation<T: ResourceKind> {
    #[getset(get = "pub")]
    namespace: Namespace,
    #[getset(get = "pub")]
    id: ResourceId,
    #[reflect(ignore)]
    phantom_data: PhantomData<T>,
}

impl<T: ResourceKind> ResourceLocation<T> {
    pub fn from_path(path: impl AsRef<Path>) -> prelude::Result<Self, ResourceLocationParseError> {
        let path = path.as_ref();

        let path = if let Some(ext) = path.extension() {
            if ext.to_str().unwrap_or_default() != T::FILE_TYPE.ext() {
                return Err(ResourceLocationParseError::InvalidPath(format!(
                    "Mismatched extension: {}, expected: {}",
                    ext.to_str().unwrap_or_default(),
                    T::FILE_TYPE.ext()
                )));
            }

            path.with_extension("")
        } else {
            path.to_path_buf()
        };

        let mut components = path.components();

        let namespace = components
            .next()
            .ok_or(ResourceLocationParseError::Empty)?
            .as_os_str()
            .to_str()
            .ok_or(ResourceLocationParseError::InvalidPath(
                path.display().to_string(),
            ))?;
        let namespace = Namespace::from_str(namespace)?;

        let path_without_namespace = components.as_path();

        let id_path = path_without_namespace
            .strip_prefix(T::ROOT_DIR)
            .map_err(|_| {
                ResourceLocationParseError::InvalidPath(format!(
                    "Mismatched root dir: {}, expected: {}",
                    path.display(),
                    T::ROOT_DIR
                ))
            })?;

        let id = id_path
            .components()
            .map(|c| {
                c.as_os_str()
                    .to_str()
                    .ok_or(ResourceLocationParseError::InvalidPath(
                        path.display().to_string(),
                    ))
            })
            .filter_map(|s| s.ok())
            .collect::<Vec<_>>()
            .join("/");
        let id = ResourceId::from_str(&id)?;

        Ok(Self {
            namespace,
            id,
            phantom_data: Default::default(),
        })
    }

    /// Returns the full path to the associated file on disk, relative to the base assets folder
    pub fn as_path(&self) -> PathBuf {
        Path::new(&self.namespace.0)
            .join(T::ROOT_DIR)
            .join(&self.id.0)
            .with_extension(T::FILE_TYPE.ext())
    }

    /// Returns the resource location as a path, without the root folder or file extension
    pub fn as_local_path(&self) -> PathBuf {
        Path::new(&self.namespace.0).join(&self.id.0)
    }

    pub fn get(&self, registry: &ResourceRegistry<T>) -> Option<Handle<T::AssetKind>> {
        registry.get(self).cloned()
    }
}

impl<T: ResourceKind> ResourceLoc for ResourceLocation<T> {
    fn new(namespace: Namespace, id: ResourceId) -> Self {
        Self {
            namespace,
            id,
            phantom_data: Default::default(),
        }
    }
}

impl<T: ResourceKind> From<ResourceLocation<T>> for AssetPath<'_> {
    fn from(value: ResourceLocation<T>) -> Self {
        AssetPath::from_path_buf(value.as_path())
    }
}

impl<T: ResourceKind> Serialize for ResourceLocation<T> {
    fn serialize<S>(&self, serializer: S) -> prelude::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T: ResourceKind> Deserialize<'de> for ResourceLocation<T> {
    fn deserialize<D>(deserializer: D) -> prelude::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

impl<T: ResourceKind> Display for ResourceLocation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.id)
    }
}

impl<T: ResourceKind> FromStr for ResourceLocation<T> {
    type Err = ResourceLocationParseError;

    fn from_str(s: &str) -> prelude::Result<Self, Self::Err> {
        Self::from_str_impl(s)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Getters, Reflect)]
pub struct AnyResourceLocation {
    #[getset(get = "pub")]
    namespace: Namespace,
    #[getset(get = "pub")]
    id: ResourceId,
}

impl AnyResourceLocation {
    /// Get only the lowest component of the id
    pub fn get_file_name(&self) -> String {
        let id = self.id.to_string();
        id.split('/').next_back().unwrap().to_string()
    }

    /// Returns the resource location as a path
    pub fn as_path(&self) -> PathBuf {
        Path::new(&self.namespace.0).join(&self.id.0)
    }
}

impl ResourceLoc for AnyResourceLocation {
    fn new(namespace: Namespace, id: ResourceId) -> Self {
        Self { namespace, id }
    }
}

impl<T: ResourceKind> From<AnyResourceLocation> for ResourceLocation<T> {
    fn from(value: AnyResourceLocation) -> Self {
        Self {
            namespace: value.namespace,
            id: value.id,
            phantom_data: Default::default(),
        }
    }
}

impl<T: ResourceKind> From<ResourceLocation<T>> for AnyResourceLocation {
    fn from(value: ResourceLocation<T>) -> Self {
        Self {
            namespace: value.namespace,
            id: value.id,
        }
    }
}

impl Display for AnyResourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.id)
    }
}

impl FromStr for AnyResourceLocation {
    type Err = ResourceLocationParseError;

    fn from_str(s: &str) -> prelude::Result<Self, Self::Err> {
        Self::from_str_impl(s)
    }
}

static DEFAULT_NAMESPACE_NAME: &str = "base";
static NAMESPACE_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9_-]+$").unwrap());
static RESOURCE_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9/_-]+$").unwrap());

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Reflect)]
#[serde(try_from = "String")]
pub struct Namespace(String);

impl Default for Namespace {
    fn default() -> Self {
        Self(DEFAULT_NAMESPACE_NAME.to_string())
    }
}

impl FromStr for Namespace {
    type Err = ResourceLocationParseError;
    fn from_str(s: &str) -> prelude::Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ResourceLocationParseError::Empty);
        }

        if !NAMESPACE_PATTERN.is_match(s) {
            let invalid_chars: Vec<char> = s
                .chars()
                .filter(|c| !matches!(c, 'a'..='z' | '0'..='9' | '-' | '_'))
                .collect();

            return Err(ResourceLocationParseError::InvalidNamespaceChars(
                s.to_string(),
                invalid_chars,
            ));
        }

        Ok(Self(s.to_string()))
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Reflect)]
#[serde(try_from = "String")]
pub struct ResourceId(String);

impl FromStr for ResourceId {
    type Err = ResourceLocationParseError;
    fn from_str(s: &str) -> prelude::Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ResourceLocationParseError::Empty);
        }

        if s.starts_with('/') || s.ends_with('/') || s.contains("//") {
            return Err(ResourceLocationParseError::InvalidPath(s.to_string()));
        }

        if !RESOURCE_PATTERN.is_match(s) {
            let invalid_chars: Vec<char> = s
                .chars()
                .filter(|c| !matches!(c, 'a'..='z' | '0'..='9' | '-' | '_' | '/'))
                .collect();

            return Err(ResourceLocationParseError::InvalidResourceChars(
                s.to_string(),
                invalid_chars,
            ));
        }

        Ok(Self(s.to_string()))
    }
}

impl Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceLocationParseError {
    #[error(
        "Error parsing {0}: Resource namespaces may only contain [a-z, 0-9, -, _]. The following characters were found: {1:?}"
    )]
    InvalidNamespaceChars(String, Vec<char>),
    #[error(
        "Error parsing {0}: Resource ids may only contain [a-z, 0-9, -, _, /]. The following characters were found: {1:?}"
    )]
    InvalidResourceChars(String, Vec<char>),
    #[error(
        "Error parsing {0}: Resource id must have a valid path (no trailing slash, no double slashes, etc."
    )]
    InvalidPath(String),
    #[error("Error parsing {0}: Resource locations may contain at most one divider character ':'")]
    MultipleDividers(String),
    #[error("Resource locations must contain at least one component")]
    Empty,
}

pub fn loc<T: ResourceKind>(s: &str) -> prelude::Result<ResourceLocation<T>, ResourceLocationParseError> {
    ResourceLocation::<T>::from_str(s)
}