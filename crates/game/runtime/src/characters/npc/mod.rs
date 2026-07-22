pub mod ai;

use crate::characters::npc::ai::ai_scene;
use crate::characters::CharacterPrototype;
use assets::resource::characters::CharacterResource;
use bevy::prelude::*;
use common::marker;
use data::prelude::*;

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