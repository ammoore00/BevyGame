use crate::dev_tools::editor::window::properties::EditorCodec;
use crate::screens::Screen;
use assets::codec::{AnimationCodec, AttackCodec, CharacterCodec};
use assets::resource::characters::{AnimationResource, AttackResource, CharacterResource};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use crossbeam::channel::{Receiver, Sender};
use data::loc::AnyResourceLocation;
use data::prelude::*;
use getset::Getters;
use std::any::TypeId;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<FileManager>();

    app.init_resource::<FileTaskChannel<CharacterCodec>>();
    app.init_resource::<FileTaskChannel<AnimationCodec>>();
    app.init_resource::<FileTaskChannel<AttackCodec>>();

    app.add_systems(
        Update,
        (
            process_file_load_results::<CharacterCodec>,
            process_file_load_results::<AnimationCodec>,
            process_file_load_results::<AttackCodec>,
        )
            .run_if(in_state(Screen::Editor)),
    );
}

#[derive(SystemParam)]
pub struct FileTaskChannelSet<'w> {
    character: ResMut<'w, FileTaskChannel<CharacterCodec>>,
    animation: ResMut<'w, FileTaskChannel<AnimationCodec>>,
    attack: ResMut<'w, FileTaskChannel<AttackCodec>>,
}
impl FileTaskChannelSet<'_> {
    fn get<Codec: EditorCodec>(&self) -> &FileTaskChannel<Codec> {
        let type_id = TypeId::of::<Codec>();
        match Codec::FILE_TYPE {
            FileKind::Character if type_id == TypeId::of::<CharacterCodec>() => {
                // SAFETY: This cast is safe because we verify the type_id above
                unsafe { &*(self.character.as_ref() as *const _ as *const FileTaskChannel<Codec>) }
            }
            FileKind::Animation if type_id == TypeId::of::<AnimationCodec>() => unsafe {
                &*(self.animation.as_ref() as *const _ as *const FileTaskChannel<Codec>)
            },
            FileKind::Attack if type_id == TypeId::of::<AttackCodec>() => unsafe {
                &*(self.attack.as_ref() as *const _ as *const FileTaskChannel<Codec>)
            },
            _ => {
                panic!(
                    "Requested codec type does not match the associated FileKind: {:?}. This is a compile time bug!",
                    Codec::FILE_TYPE
                )
            }
        }
    }
}

#[derive(Resource)]
struct FileTaskChannel<T: EditorCodec> {
    sender: Sender<(EditorFileComponent, EditorFileResult<T>)>,
    receiver: Receiver<(EditorFileComponent, EditorFileResult<T>)>,
}
impl<T: EditorCodec> Default for FileTaskChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam::channel::unbounded();
        Self { sender, receiver }
    }
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
        channel_set: &FileTaskChannelSet,
    ) {
        let file = EditorFile { loc, kind };

        match kind {
            FileKind::Character => {
                self.open_typed::<CharacterCodec>(file, set_active, &channel_set.character)
            }
            FileKind::Animation => {
                self.open_typed::<AnimationCodec>(file, set_active, &channel_set.animation)
            }
            FileKind::Attack => {
                self.open_typed::<AttackCodec>(file, set_active, &channel_set.attack)
            }
        }
    }

    fn open_typed<Codec>(
        &mut self,
        file: EditorFile,
        set_active: bool,
        channel: &FileTaskChannel<Codec>,
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

        let mut path = PathBuf::from("assets/");
        path.push(file.loc().as_path());

        let task_pool = IoTaskPool::get();
        let sender = channel.sender.clone();

        // Spawn the async task
        task_pool
            .spawn(async move {
                let result = load_codec::<Codec>(&path).await;

                info!("Loaded file {}", path.to_str().unwrap());

                sender.send((EditorFileComponent(file), EditorFileResult(result)))
            })
            .detach();
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
            Err(FileManagerError::Io(
                format!("File not found: {}", file.loc),
                std::io::Error::from(std::io::ErrorKind::NotFound),
            ))
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
    ron::from_str(&contents).map_err(|err| {
        FileManagerError::Decode(
            format!("Failed to deserialize file: {:?}", path),
            Box::new(err),
        )
    })
}

fn process_file_load_results<Codec: EditorCodec>(
    file_query: Query<(Entity, &EditorFileComponent, &EditorFileResult<Codec>)>,
    channel_set: FileTaskChannelSet,
    mut commands: Commands,
) {
    let channel = channel_set.get::<Codec>();

    // Spawn any entities waiting in the channel
    for data in channel.receiver.try_iter() {
        info!("Spawning entity from channel: {}", data.0.0.loc());
        commands.spawn(data);
    }

    // Process any spawned entities
    // Note that entities spawned above will have to wait until the next frame to be processed
    for (entity, file, result) in file_query {
        match &result.0 {
            // If the file was loaded successfully, insert the content and remove the result
            Ok(codec) => {
                info!("File loaded successfully: {}", file.0.loc());
                commands
                    .entity(entity)
                    .insert(Codec::FILE_CONTENT_FN(codec.clone()));
                commands.entity(entity).remove::<EditorFileResult<Codec>>();
            }
            Err(FileManagerError::Io(_, err)) => {
                match err.kind() {
                    // If the file did not exist, insert a default content and remove the result
                    std::io::ErrorKind::NotFound => {
                        info!("Creating new file for: {}", file.0.loc());
                        commands
                            .entity(entity)
                            .insert(Codec::FILE_CONTENT_FN(Codec::default()));
                        commands.entity(entity).remove::<EditorFileResult<Codec>>();
                    }
                    // Otherwise, log the error and despawn the entity
                    _ => {
                        error!("Failed to load file: {:?}", err);
                        commands.entity(entity).despawn()
                    }
                }
            }
            // Otherwise, log the error and despawn the entity
            Err(err) => {
                error!("Failed to load file: {:?}", err);
                commands.entity(entity).despawn()
            }
        }
    }
}

#[derive(Component, Clone)]
pub struct EditorFileComponent(pub EditorFile);

#[derive(Component, Debug)]
pub struct EditorFileResult<Codec: EditorCodec>(Result<Codec, FileManagerError>);

#[derive(Component, Debug, Clone)]
pub enum EditorFileContent {
    Character(CharacterCodec),
    Animation(AnimationCodec),
    Attack(AttackCodec),
}
macro_rules! get_variant {
    ($fn_name:ident, $return_type:ty, $variant:ident) => {
        /// If this is a matching file, return the stored codec.
        /// # Panics
        /// This function will panic if the file kind does not match the expected kind.
        pub fn $fn_name(&self) -> $return_type {
            match self {
                EditorFileContent::$variant(codec) => codec.clone(),
                _ => panic!(
                    "Expected file kind {:?}, but got {:?}",
                    stringify!($variant),
                    self.name()
                ),
            }
        }
    };
}
impl EditorFileContent {
    get_variant!(get_character_codec, CharacterCodec, Character);
    get_variant!(get_animation_codec, AnimationCodec, Animation);
    get_variant!(get_attack_codec, AttackCodec, Attack);

    fn name(&self) -> String {
        match self {
            EditorFileContent::Character(_) => "Character".to_string(),
            EditorFileContent::Animation(_) => "Animation".to_string(),
            EditorFileContent::Attack(_) => "Attack".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters)]
pub struct EditorFile {
    #[get = "pub"]
    loc: AnyResourceLocation,
    #[get = "pub"]
    kind: FileKind,
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
