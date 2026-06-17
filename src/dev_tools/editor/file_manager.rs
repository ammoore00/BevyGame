use crate::data::{AnyResourceLocation, ResourceKind, ResourceLocation};
use crate::datagen_api::animation::AnimationResource;
use crate::datagen_api::assets::CharacterResource;
use crate::datagen_api::attack::AttackResource;
use bevy::prelude::*;
use getset::Getters;
use std::fmt::Debug;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<FileManager>();
}

#[derive(Resource, Debug, Default, Getters)]
pub struct FileManager {
    #[getset(get = "pub")]
    open_files: Vec<EditorFile>,
    #[getset(get = "pub")]
    active_index: Option<usize>,
}
impl FileManager {
    pub fn open(&mut self, loc: AnyResourceLocation, kind: FileKind) {
        let file = EditorFile {
            loc,
            kind,
        };

        // If the file is already open, just set it as active
        // Otherwise, open the file, then set it as active
        if let Some(pos) = self.open_files.iter().position(|f| f == &file) {
            info!("File {} already open, setting as active", file.loc);
            self.active_index = Some(pos);
        } else {
            info!("Opening file {}", file.loc);
            self.active_index = Some(self.open_files.len());
            self.open_files.push(file);
        }
    }

    pub fn close(&mut self, file: EditorFile) {
        if let Some(pos) = self.open_files.iter().position(|f| f == &file) {
            // If the active file is being closed
            if self.active_index == Some(pos) {
                // If it's not the first file, move to the next file lower
                if pos > 0 {
                    self.active_index = Some(pos - 1);
                }
                // If it is the first file, and there are more files open,
                // just keep the pos where it is, since it will become the next higher file
                else if self.open_files.len() > 1 {
                    self.active_index = Some(pos);
                }
                // If this was the only open file, set active file to None
                else {
                    self.active_index = None;
                }
            }
            // If the active file is higher than the closed file, move it down one
            else if self.active_index > Some(pos) {
                self.active_index = self.active_index.map(|i| i - 1);
            }

            self.open_files.remove(pos);
        }
    }
    
    pub fn get_active_file(&self) -> Option<&EditorFile> {
        self.active_index.map(|i| &self.open_files[i])
    }
    
    pub fn set_active_file(&mut self, file: &EditorFile) -> Result<(), FileManagerError> {
        if let Some(pos) = self.open_files.iter().position(|f| f == file) {
            self.active_index = Some(pos);
            Ok(())
        } else {
            Err(FileManagerError::FileNotFound)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FileManagerError {
    #[error("File not found")]
    FileNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters)]
pub struct EditorFile {
    #[get = "pub"]
    loc: AnyResourceLocation,
    kind: FileKind,
}
impl EditorFile {
    pub fn name(&self) -> String {
        self.loc.get_file_name()
    }
    
    /// Get the resource location as a typed resource location
    /// # Panics
    /// If the file kind does not match the generic type provided
    fn loc_typed<T: EditorResourceKind>(&self) -> ResourceLocation<T> {
        guard_kind::<T>(self.kind);
        self.loc.clone().into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    Character,
    Animation,
    Attack,
}
impl FileKind {
    pub(crate) fn from_resource_kind<T: EditorResourceKind>() -> Self {
        T::FILE_KIND
    }
}

/// Trait for resource kinds supported by the editor.
pub trait EditorResourceKind: ResourceKind {
    const FILE_KIND: FileKind;
}

impl EditorResourceKind for CharacterResource {
    const FILE_KIND: FileKind = FileKind::Character;
}
impl EditorResourceKind for AnimationResource {
    const FILE_KIND: FileKind = FileKind::Animation;
}
impl EditorResourceKind for AttackResource {
    const FILE_KIND: FileKind = FileKind::Attack;
}

/// Guard that checks if the file kind matches the ResourceKind type.
///
/// # Panics
/// This function will panic if the file kind does not match the ResourceKind type.
/// This is intended as a guard against compile time bugs.
fn guard_kind<T: EditorResourceKind>(kind: FileKind) {
    panic!(
        "File kind mismatch! Expected {:?}, but got {:?}. This is a compile-time bug!",
        T::FILE_KIND,
        kind
    );
}