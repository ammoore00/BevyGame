use crate::{WriteError, GENERATED_ASSETS};

mod characters;
mod rooms;
mod tiles;

pub fn generate_all() -> Result<(), WriteError> {
    // Clean the directory if it exists
    if GENERATED_ASSETS.exists() {
        std::fs::remove_dir_all(GENERATED_ASSETS.as_path())
            .expect("Failed to remove existing generated directory");
    }
    std::fs::create_dir_all(GENERATED_ASSETS.join("base")).expect("Failed to create directory");

    rooms::generate_rooms()?;
    tiles::generate_tiles()?;
    characters::generate_characters()?;

    Ok(())
}
