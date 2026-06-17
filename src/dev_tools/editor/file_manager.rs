use std::fmt::Debug;
use bevy::prelude::*;
use crate::data::{AnyResourceLocation, ResourceKind, ResourceLocation};
use crate::datagen_api::animation::AnimationResource;
use crate::datagen_api::assets::CharacterResource;
use crate::datagen_api::attack::AttackResource;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<FileManager>();
}

#[derive(Resource, Debug, Default)]
pub struct FileManager {
    open_files: Vec<File>,
    active_file: Option<usize>,
}
impl FileManager {
    pub fn open(&mut self, loc: AnyResourceLocation, kind: FileKind) {
        info!("Opened file: {} // File kind, {:?}", loc, kind);

        self.open_files.push(File {
            loc,
            kind,
        });
    }
}

#[derive(Debug, Clone)]
struct File {
    loc: AnyResourceLocation,
    kind: FileKind,
}
impl File {
    fn loc_typed<T: EditorResourceKind>(&self) -> ResourceLocation<T> {
        guard_kind::<T>(self.kind);

        self.loc.clone().into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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