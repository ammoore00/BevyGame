use crate::asset_tracking::LoadResource;
use crate::game::grid::coords::{SCREEN_Z_SCALE, TileCoords};
pub(crate) use crate::game::grid::tile::TileAssets;
use bevy::prelude::*;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub mod coords;
pub mod tile;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TileAssets>();

    app.add_plugins(coords::plugin);
}

#[derive(Component)]
pub struct Grid;

pub fn grid(_tile_map: Arc<RwLock<BTreeMap<TileCoords, Entity>>>, scale: f32) -> impl Bundle {
    (
        Grid,
        Transform::from_scale(Vec2::splat(scale).extend(SCREEN_Z_SCALE)),
        InheritedVisibility::default(),
    )
}
