pub mod ai;

use crate::data::ResourceLocation;
use crate::datagen_api::assets::CharacterResource;
use crate::marker;
use bevy::prelude::*;
use crate::datagen_api::ai::ai_scene;
use crate::game::character::CharacterPrototype;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(ai::plugin);
}

pub fn npc_bundle(
    data_loc: ResourceLocation<CharacterResource>,
    position: Vec3,
) -> impl Scene {
    bsn! [
        Npc
        ai_scene()
        @CharacterPrototype {
            @position,
            @data_loc,
        }
    ]
}

marker!(Npc);