
use data::prelude::*;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tracing::info;
use crate::defs::generate_all;

mod character;
pub mod room;
mod sprite;
pub mod tile;
mod defs;

static GENERATED_ASSETS: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(Path::new("../../assets/generated")));
pub static ROOT: LazyLock<PathBuf> = LazyLock::new(|| GENERATED_ASSETS.join("base"));

fn main() {
    generate_all().expect("Failed to generate assets")
}

pub fn create_dir(dir: impl AsRef<Path>) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(ROOT.join(dir))
}

pub fn write_data<R, D>(loc: ResourceLocation<R>, codec: &D) -> Result<(), WriteError>
where
    R: ResourceKind,
    D: ?Sized + Serialize,
{
    let serialized = ron::ser::to_string_pretty(&codec, ron::ser::PrettyConfig::default())?;
    let serialized = compact_integer_arrays(&serialized);

    // Create all parent directories
    let path = GENERATED_ASSETS.join(loc.as_path());
    if let Some(parent) = path.parent() {
        info!("Creating directory: {}", parent.display());
        std::fs::create_dir_all(parent).map_err(|err| WriteError::io(&loc, err))?;
    }

    let file = match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            info!("Skipping existing file: {}", path.display());
            return Ok(());
        }
        Err(err) => return Err(WriteError::io(&loc, err)),
    };

    let mut writer = std::io::BufWriter::new(file);
    writer
        .write_all(serialized.as_bytes())
        .map_err(|err| WriteError::io(&loc, err))?;
    Ok(())
}

fn compact_integer_arrays(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut stack: Vec<usize> = Vec::new();
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    let mut in_string = false;
    let mut escaped = false;

    for (i, &b) in bytes.iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }

        match b {
            b'"' => in_string = true,
            b'[' => stack.push(i),
            b']' => {
                if let Some(start) = stack.pop() {
                    let end = i + 1;
                    let inner = &input[start + 1..i];

                    if is_simple_integer_array(inner) {
                        let compact = compact_array_inner(inner);
                        replacements.push((start, end, format!("[{}]", compact)));
                    }
                }
            }
            _ => {}
        }
    }

    let mut out = input.to_string();
    for (start, end, replacement) in replacements.into_iter().rev() {
        out.replace_range(start..end, &replacement);
    }
    out
}

fn is_simple_integer_array(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    trimmed
        .chars()
        .all(|c| c.is_ascii_digit() || c == ',' || c.is_ascii_whitespace() || c == '-')
}

fn compact_array_inner(text: &str) -> String {
    text.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("Error when writing resource {loc} to {path}: {err}")]
    File {
        loc: String,
        path: String,
        err: std::io::Error,
    },
    #[error("I/O error: {0}")]
    OtherIo(#[from] std::io::Error),
    #[error("RON serialize error: {0}")]
    Ron(#[from] ron::Error),
}
impl WriteError {
    fn io<T: ResourceKind>(loc: &ResourceLocation<T>, err: std::io::Error) -> Self {
        Self::File {
            loc: loc.to_string(),
            path: loc.as_path().to_string_lossy().parse().unwrap(),
            err,
        }
    }
}
