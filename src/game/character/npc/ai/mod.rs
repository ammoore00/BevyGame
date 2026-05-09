mod pathfinding;

use crate::data;
use crate::data::{ResourceFileType, ResourceLocation};
use crate::define_data_resource;
use crate::define_resource;
use crate::game::character::npc::ai::pathfinding::pathfinder_bundle;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::data::loader::RonAssetLoader;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(pathfinding::plugin);

    app.init_asset::<AiState>();
    app.init_asset_loader::<RonAssetLoader<AiStateCodec, AiState>>();

    app.init_asset::<AiGraph>();
    app.init_asset_loader::<RonAssetLoader<AiGraphCodec, AiGraph>>();
}

pub(super) fn ai_bundle() -> impl Bundle {
    (
        pathfinder_bundle(),
    )
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
struct AiNode {
    state: ResourceLocation<AiStateResource>,
    kind: AiNodeKind,
}

#[derive(Debug, Clone)]
enum AiNodeKind {
    Selector {
        children: Vec<AiNodeKind>,
    },
    Sequence {
        sequence: Vec<AiNodeKind>,
    },
}