pub mod ai;

use crate::data::ResourceLocation;
use crate::datagen_api::assets::CharacterResource;
use crate::game::character::npc::ai::ai_bundle;
use crate::game::character::{character_bundle, CharacterBuilderContext};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(ai::plugin);
}

pub fn npc_bundle(
    data_loc: ResourceLocation<CharacterResource>,
    position: Vec3,
    scale: f32,
    context: &CharacterBuilderContext,
) -> impl Bundle {
    let character_data = character_bundle(data_loc, position, scale, context);
    let ai_data = ai_bundle();

    (
        Npc,
        ai_data,
        character_data,
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Npc;