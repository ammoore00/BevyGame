use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use crate::room::generate_rooms;
use crate::tiles::generate_tiles;

pub mod room;
pub mod tiles;

pub static ROOT: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(Path::new("../assets/generated")));
pub static ROOT_BASE: LazyLock<PathBuf> = LazyLock::new(|| ROOT.join("base"));

fn main() {
    std::fs::create_dir_all(ROOT.join("base")).expect("Failed to create directory");
    generate_tiles().expect("Failed to generate tiles");
    generate_rooms().expect("Failed to generate rooms");
}