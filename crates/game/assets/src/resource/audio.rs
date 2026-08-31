use crate::loader::LoaderJobManager;
use bevy::prelude::*;
use data::prelude::ResourceFileType;
use data::resource::resource_kind;

pub(super) fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<AudioResource>();
}

#[resource_kind(path = "audio", asset_kind = AudioSource, file_type = ResourceFileType::Audio)]
pub struct AudioResource;
