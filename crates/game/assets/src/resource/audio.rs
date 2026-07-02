use crate::loader::LoaderJobManager;
use bevy::prelude::*;
use data::define_resource;
use data::prelude::ResourceFileType;

pub(super) fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<AudioResource>();
}

define_resource!(Audio, "audio", AudioSource, ResourceFileType::Audio);