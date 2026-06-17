use crate::data::{AnyResourceLocation, ResourceKind, ResourceLocation};
use crate::datagen_api::animation::{AnimationCodec, AnimationResource};
use crate::datagen_api::assets::{CharacterCodec, CharacterResource};
use crate::datagen_api::attack::{AttackCodec, AttackResource};
use crate::dev_tools::editor::window::properties::EditorCodec;
use bevy::prelude::*;
use getset::Getters;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use bevy::tasks::IoTaskPool;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<FileManager>();

    app.add_systems(
        Update,
        (
            process_file_load_results::<CharacterCodec>,
            process_file_load_results::<AnimationCodec>,
            process_file_load_results::<AttackCodec>,
        )
            .run_if(in_state(Screen::Editor))
    );
}

#[derive(Resource, Debug, Default, Getters)]
pub struct FileManager {
    #[getset(get = "pub")]
    open_files: Vec<EditorFile>,
    #[getset(get = "pub")]
    active_index: Option<usize>,
}
impl FileManager {
    pub fn open(
        &mut self,
        loc: AnyResourceLocation,
        kind: FileKind,
        set_active: bool,
    ) {
        let file = EditorFile {
            loc,
            kind,
        };

        match kind {
            FileKind::Character => self.open_typed::<CharacterCodec>(file, set_active),
            FileKind::Animation => self.open_typed::<AnimationCodec>(file, set_active),
            FileKind::Attack => self.open_typed::<AttackCodec>(file, set_active),
        }
    }

    fn open_typed<Codec>(
        &mut self,
        file: EditorFile,
        set_active: bool,
    ) where
        Codec: EditorCodec,
    {
        // If the file is already open, just set it as active
        // Otherwise, open the file, then set it as active
        if let Some(pos) = self.open_files.iter().position(|f| f == &file) {
            if set_active {
                info!("File {} already open, setting as active", file.loc());
                self.active_index = Some(pos);
            }
        } else {
            info!("Opening file {}", file.loc());
            if set_active || self.active_index.is_none() {
                self.active_index = Some(self.open_files.len());
            }
            self.open_files.push(file.clone());
        }

        let task_pool = IoTaskPool::get();
        let mut path = PathBuf::from("assets/");
        path.push(file.loc().as_path());

        // Spawn the async task
        task_pool.spawn(async move {
            let result = load_codec::<Codec>(&path).await;

            info!("Loaded file {}", path.to_str().unwrap());

            (
                EditorFileComponent(file),
                EditorFileResult(result),
            )
        }).detach();
    }

    pub fn close(&mut self, file: &EditorFile) {
        if let Some(pos) = self.open_files.iter().position(|f| f == file) {
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

    pub fn is_file_open(&self, file: &EditorFile) -> bool {
        self.open_files.iter().any(|f| f == file)
    }

    pub fn active_file(&self) -> Option<&EditorFile> {
        self.active_index.map(|i| &self.open_files[i])
    }

    pub fn set_active_file(&mut self, file: &EditorFile) -> Result<(), FileManagerError> {
        if let Some(pos) = self.open_files.iter().position(|f| f == file) {
            self.active_index = Some(pos);
            Ok(())
        } else {
            Err(FileManagerError::Io(format!("File not found: {}", file.loc), std::io::Error::from(std::io::ErrorKind::NotFound)))
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FileManagerError {
    #[error("I/O error: {0}")]
    Io(String, #[source] std::io::Error),
    #[error("Decoding error: {0}")]
    Decode(String, #[source] Box<ron::error::SpannedError>),
}

async fn load_codec<Codec>(path: &Path) -> Result<Codec, FileManagerError>
where
    Codec: EditorCodec,
{
    let contents = std::fs::read_to_string(path)
        .map_err(|err| FileManagerError::Io(format!("Failed to read file: {:?}", path), err))?;
    ron::from_str(&contents)
        .map_err(|err| FileManagerError::Decode(format!("Failed to deserialize file: {:?}", path), Box::new(err)))
}

fn process_file_load_results<Codec: EditorCodec>(
    file_query: Query<
        (
            Entity,
            &EditorFileResult<Codec>,
        ),
        With<EditorFileComponent>,
    >,
    mut commands: Commands,
) {
    for (entity, result) in file_query {
        match &result.0 {
            // If the file was loaded successfully, insert the content and remove the result
            Ok(codec) => {
                commands.entity(entity).insert(EditorFileContent(codec.clone()));
                commands.entity(entity).remove::<EditorFileResult<Codec>>();
            },
            Err(FileManagerError::Io(_, err)) => {
                match err.kind() {
                    // If the file did not exist, insert a default content and remove the result
                    std::io::ErrorKind::NotFound => {
                        commands.entity(entity).insert(EditorFileContent(Codec::default()));
                        commands.entity(entity).remove::<EditorFileResult<Codec>>();
                    },
                    // Otherwise, log the error and despawn the entity
                    _ => {
                        error!("Failed to load file: {:?}", err);
                        commands.entity(entity).despawn()
                    },
                }
            },
            // Otherwise, log the error and despawn the entity
            Err(err) => {
                error!("Failed to load file: {:?}", err);
                commands.entity(entity).despawn()
            }
        }
    }
}

#[derive(Component, Clone)]
pub struct EditorFileComponent(EditorFile);
#[derive(Component, Debug)]
pub struct EditorFileResult<Codec: EditorCodec>(Result<Codec, FileManagerError>);
#[derive(Component, Debug, Clone)]
pub struct EditorFileContent<Codec: EditorCodec>(pub Codec);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters)]
pub struct EditorFile {
    #[get = "pub"]
    loc: AnyResourceLocation,
    #[get = "pub"]
    kind: FileKind,
}
impl EditorFile {
    pub fn _name(&self) -> String {
        self.loc.get_file_name()
    }

    /// Get the resource location as a typed resource location
    /// # Panics
    /// If the file kind does not match the generic type provided
    pub fn loc_typed<T: EditorResourceKind>(&self) -> ResourceLocation<T> {
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
    pub fn from_resource_kind<T: EditorResourceKind>() -> Self {
        T::FILE_KIND
    }
}

/// Trait for resource kinds supported by the editor.
pub trait EditorResourceKind: ResourceKind {
    type Codec: EditorCodec;
    const FILE_KIND: FileKind;
}

impl EditorResourceKind for CharacterResource {
    type Codec = CharacterCodec;
    const FILE_KIND: FileKind = FileKind::Character;
}
impl EditorResourceKind for AnimationResource {
    type Codec = AnimationCodec;
    const FILE_KIND: FileKind = FileKind::Animation;
}
impl EditorResourceKind for AttackResource {
    type Codec = AttackCodec;
    const FILE_KIND: FileKind = FileKind::Attack;
}

/// Guard that checks if the file kind matches the ResourceKind type.
///
/// # Panics
/// This function will panic if the file kind does not match the ResourceKind type.
/// This is intended as a guard against compile time bugs.
fn guard_kind<T: EditorResourceKind>(kind: FileKind) {
    if kind != T::FILE_KIND {
        panic!(
            "File kind mismatch! Expected {:?}, but got {:?}. This is a compile-time bug!",
            T::FILE_KIND,
            kind
        );
    }
}