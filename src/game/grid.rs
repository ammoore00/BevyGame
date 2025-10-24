use crate::ReflectResource;
use bevy::app::App;
use bevy::asset::{Asset, AssetServer, Handle};
use bevy::image::Image;
use bevy::math::Vec2;
use bevy::prelude::{Bundle, Component, FromWorld, Reflect, Resource, Sprite, Transform, World};
use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TileDebugAssets>();
}

#[derive(Component)]
struct Grid;

pub fn grid(
    tile_debug_assets: &TileDebugAssets
) -> impl Bundle {
    (
        Grid,
        Sprite::from(tile_debug_assets.grass.clone()),
        Transform::from_scale(Vec2::splat(6.0).extend(1.0))
    )
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct TileDebugAssets {
    #[dependency]
    grass: Handle<Image>,
}

impl FromWorld for TileDebugAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        TileDebugAssets {
            grass: assets.load("images/grass.png"),
        }
    }
}