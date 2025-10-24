use crate::ReflectResource;
use bevy::prelude::*;
use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TileDebugAssets>();
}

#[derive(Component)]
struct Grid;

pub fn grid(
    tile_debug_assets: TileDebugAssets
) -> impl Bundle {
    (
        Grid,
        Transform::from_scale(Vec2::splat(6.0).extend(1.0)),
        InheritedVisibility::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            let size_x = 2;
            let size_z = 2;

            for i in -size_x..=size_x {
                for k in -size_z..=size_z {
                    parent.spawn((
                        tile(
                            IVec3::from([i, 0, k]),
                            &tile_debug_assets
                        ),
                    ));
                }
            }
        })),
    )
}

pub const TILE_WIDTH: i32 = 32;
pub const TILE_HEIGHT: i32 = 16;

#[derive(Component)]
struct Tile(pub IVec3);

pub fn tile(
    world_coords: IVec3,
    tile_debug_assets: &TileDebugAssets,
) -> impl Bundle {
    let screen_x = (world_coords.x - world_coords.z) * TILE_WIDTH / 2;
    let screen_y = (world_coords.x + world_coords.z) * TILE_HEIGHT / 2;

    let screen_position = Vec2::new(screen_x as f32, screen_y as f32).extend(0.0);

    (
        Tile(world_coords),
        Sprite::from(tile_debug_assets.grass.clone()),
        Transform::from_translation(screen_position),
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