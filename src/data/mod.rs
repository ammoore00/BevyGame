pub mod registry;
pub mod sprite;
pub mod loader;

use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use regex::Regex;
use std::str::FromStr;
use std::sync::LazyLock;
use bevy::prelude::*;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

pub fn plugin(app: &mut App) {
    app.add_plugins((sprite::plugin, loader::plugin));
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct ResourceLocation<T: ResourceType> {
    namespace: Namespace,
    id: ResourceId,
    phantom_data: PhantomData<T>,
}
impl<T: ResourceType> Serialize for ResourceLocation<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de, T: ResourceType> Deserialize<'de> for ResourceLocation<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

impl<T: ResourceType> Display for ResourceLocation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.id)
    }
}

impl<T: ResourceType> ResourceLocation<T> {
    pub fn new(namespace: Namespace, id: ResourceId) -> Self {
        Self { namespace, id, phantom_data: Default::default() }
    }

    pub fn as_path(&self) -> PathBuf {
        Path::new(&self.namespace.0)
            .join(T::root_dir())
            .join(&self.id.0)
            .with_extension(T::file_type().to_string())
    }
}

impl<T: ResourceType> FromStr for ResourceLocation<T> {
    type Err = ResourceLocationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

pub trait ResourceType: Reflect + Clone + Hash + Eq + Send + Sync + 'static {
    type AssetType: Asset + Send + Sync + 'static;

    fn root_dir() -> &'static str;
    fn file_type() -> ResourceFileType;
}

pub enum ResourceFileType {
    Image,
    Audio,
    Font,
    Data,
    Other(String),
}
impl Display for ResourceFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceFileType::Image => write!(f, "png"),
            ResourceFileType::Audio => write!(f, "ogg"),
            ResourceFileType::Font => write!(f, "ttf"),
            ResourceFileType::Data => write!(f, "ron"),
            ResourceFileType::Other(s) => write!(f, "{}", s),
        }
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
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ResourceLocationParseError::Empty);
        }

        if !NAMESPACE_PATTERN.is_match(s) {
            let invalid_chars: Vec<char> = s
                .chars()
                .filter(|c| !matches!(c, 'a'..='z' | '0'..='9' | '-' | '_'))
                .collect();

            return Err(ResourceLocationParseError::InvalidNamespaceChars(s.to_string(), invalid_chars));
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
    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

            return Err(ResourceLocationParseError::InvalidResourceChars(s.to_string(), invalid_chars));
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
    #[error("Error parsing {0}: Resource namespaces may only contain [a-z, 0-9, -, _]. The following characters were found: {1:?}")]
    InvalidNamespaceChars(String, Vec<char>),
    #[error("Error parsing {0}: Resource ids may only contain [a-z, 0-9, -, _, /]. The following characters were found: {1:?}")]
    InvalidResourceChars(String, Vec<char>),
    #[error("Error parsing {0}: Resource id must have a valid path (no trailing slash, no double slashes, etc.")]
    InvalidPath(String),
    #[error("Error parsing {0}: Resource locations may contain at most one divider character ':'")]
    MultipleDividers(String),
    #[error("Resource locations must contain at least one component")]
    Empty,
}