pub mod ai;

use crate::data::loc::ResourceLocation;
use crate::data::prelude::*;
use crate::game::character::npc::ai::ai_scene;
use crate::game::character::CharacterPrototype;
use crate::marker;
use bevy::prelude::*;

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