use regex::Regex;
use std::str::FromStr;

pub struct ResourceLocation {
    namespace: Namespace,
    id: ResourceId,
}

pub struct Namespace(String);
impl Default for Namespace {
    fn default() -> Self {
        Self("game".to_string())
    }
}
impl FromStr for Namespace {
    type Err = ResourceLocationParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_resource_string(s)?;
        Ok(Self(s.to_string()))
    }
}

pub struct ResourceId(String);
impl FromStr for ResourceId {
    type Err = ResourceLocationParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_resource_string(s)?;
        Ok(Self(s.to_string()))
    }
}

fn validate_resource_string(s: &str) -> Result<(), ResourceLocationParseError> {
    if s.is_empty() {
        return Err(ResourceLocationParseError::Empty);
    }

    let valid_pattern = Regex::new(r"^[a-z0-9_-]+$").unwrap();
    if !valid_pattern.is_match(s) {
        let invalid_chars: Vec<char> = s
            .chars()
            .filter(|c| !matches!(c, 'a'..='z' | '0'..='9' | '-' | '_'))
            .collect();
        return Err(ResourceLocationParseError::InvalidChars(invalid_chars));
    }

    Ok(())
}

impl ResourceLocation {
    pub fn new(namespace: Namespace, id: ResourceId) -> Self {
        Self { namespace, id }
    }
}

impl FromStr for ResourceLocation {
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
            _ => Err(ResourceLocationParseError::MultipleDividers),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceLocationParseError {
    #[error("Resource locations may only contain [a-z, 0-9, -, _]. The following characters were found: {0:?}")]
    InvalidChars(Vec<char>),
    #[error("Resource locations may contain at most one divider character ':'")]
    MultipleDividers,
    #[error("Resource locations must contain at least one component")]
    Empty,
}