pub mod pathfinding;

use crate::data::loader::RonAssetLoader;
use crate::game::character::npc::ai::pathfinding::pathfinder_scene;
use bevy::prelude::*;
use game_data::prelude::*;
use serde::{Deserialize, Serialize};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(pathfinding::plugin);

    app.init_asset::<AiState>();
    app.init_asset_loader::<RonAssetLoader<AiStateCodec, AiState>>();

    app.init_asset::<AiGraph>();
    app.init_asset_loader::<RonAssetLoader<AiGraphCodec, AiGraph>>();
}

pub(super) fn ai_scene() -> impl Scene {
    bsn! [
        pathfinder_scene()
    ]
}

define_data_resource!(AiGraph, "characters/ai/graphs", AiGraph);
#[derive(Asset, Debug, Clone, TypePath)]
pub struct AiGraph {

}
impl From<AiGraphCodec> for AiGraph {
    fn from(_codec: AiGraphCodec) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AiGraphCodec {
    pub format: u8,
    pub states: Vec<ResourceLocation<AiStateResource>>,
}

define_data_resource!(AiState, "characters/ai/states", AiState);
#[derive(Asset, Debug, Clone, TypePath)]
pub struct AiState {

}
impl From<AiStateCodec> for AiState {
    fn from(_codec: AiStateCodec) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AiStateCodec {
    pub format: u8,
}

#[derive(Debug, Clone)]
struct _AiNode {
    state: ResourceLocation<AiStateResource>,
    kind: _AiNodeKind,
}

#[derive(Debug, Clone)]
enum _AiNodeKind {
    Selector {
        children: Vec<_AiNodeKind>,
    },
    Sequence {
        sequence: Vec<_AiNodeKind>,
    },
}