use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use serde::Serialize;
use bevy_game_2d::data::{ResourceLocation, ResourceType};
use crate::room::generate_rooms;
use crate::tiles::generate_tiles;

pub mod room;
pub mod tiles;

static ROOT_GENERATED: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(Path::new("../assets/generated")));
pub static ROOT: LazyLock<PathBuf> = LazyLock::new(|| ROOT_GENERATED.join("base"));

fn main() {
    std::fs::create_dir_all(ROOT_GENERATED.join("base")).expect("Failed to create directory");
    generate_tiles().expect("Failed to generate tiles");
    generate_rooms().expect("Failed to generate rooms");
}

pub fn create_dir(dir: impl AsRef<Path>) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(ROOT.join(dir))
}

pub fn write_data<R, D>(
    loc: ResourceLocation<R>,
    codec: &D
) -> Result<(), WriteError>
where
    R: ResourceType,
    D: ?Sized + Serialize
{
    let serialized = ron::ser::to_string_pretty(&codec, ron::ser::PrettyConfig::default())?;
    
    let file = std::fs::File::create(ROOT_GENERATED.join(loc.as_path()))?;
    let mut writer = std::io::BufWriter::new(file);
    writer.write_all(serialized.as_bytes())?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RON serialize error: {0}")]
    Ron(#[from] ron::Error),
}