use std::ops::Deref;
use crate::ReflectResource;
use bevy::prelude::*;
use crate::asset_tracking::LoadResource;
use crate::game::grid::coords::{ScreenCoords, TileCoords, TilePosition, WorldPosition};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TileDebugAssets>();

    app.add_plugins(coords::plugin);
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
struct Tile;

pub fn tile(
    tile_coords: impl Into<TileCoords> + Clone,
    tile_debug_assets: &TileDebugAssets,
) -> impl Bundle {
    (
        Tile,
        TilePosition(tile_coords.clone().into()),
        Sprite::from(tile_debug_assets.grass.clone()),
        Transform::from_translation(Into::<ScreenCoords>::into(tile_coords.into()).extend(0.0)),
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

pub mod coords {
    use std::ops::Deref;
    use bevy::prelude::*;
    use crate::game::grid::{TILE_HEIGHT, TILE_WIDTH};

    pub(super) fn plugin(app: &mut App) {
        app.add_systems(PreUpdate,
            (
                convert_world_to_screen_coords,
                convert_tile_to_screen_coords
            ));
    }

    #[derive(Component, Debug)]
    pub struct WorldPosition(pub WorldCoords);
    pub fn convert_world_to_screen_coords(
        mut query: Query<(&WorldPosition, &mut Transform)>,
    ) {
        for (world_position, mut transform) in query.iter_mut() {
            let screen_coords = ScreenCoords::from(&world_position.0);
            transform.translation = screen_coords.extend(0.0);
        }
    }

    #[derive(Component, Debug)]
    pub struct TilePosition(pub TileCoords);
    pub fn convert_tile_to_screen_coords(
        mut query: Query<(&TilePosition, &mut Transform)>,
    ) {
        for (tile_position, mut transform) in query.iter_mut() {
            let screen_coords = ScreenCoords::from(&tile_position.0);
            transform.translation = screen_coords.extend(0.0);
        }
    }

    #[derive(Debug)]
    pub struct TileCoords(pub IVec3);
    impl From<WorldCoords> for TileCoords {
        fn from(value: WorldCoords) -> Self {
            Self(value.0.as_ivec3())
        }
    }
    impl From<&WorldCoords> for TileCoords {
        fn from(value: &WorldCoords) -> Self {
            Self(value.0.as_ivec3())
        }
    }
    impl From<IVec3> for TileCoords  {
        fn from(value: IVec3) -> Self {
            TileCoords(value)
        }
    }
    impl Deref for TileCoords {
        type Target = IVec3;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug)]
    pub struct WorldCoords(pub Vec3);
    impl From<TileCoords> for WorldCoords {
        fn from(value: TileCoords) -> Self {
            Self(value.0.as_vec3())
        }
    }
    impl From<&TileCoords> for WorldCoords {
        fn from(value: &TileCoords) -> Self {
            Self(value.0.as_vec3())
        }
    }
    impl From<Vec3> for WorldCoords  {
        fn from(value: Vec3) -> Self {
            WorldCoords(value)
        }
    }
    impl Deref for WorldCoords {
        type Target = Vec3;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug)]
    pub struct ScreenCoords(pub Vec2);
    impl From<WorldCoords> for ScreenCoords {
        fn from(value: WorldCoords) -> Self {
            let screen_x = (value.x - value.z) * TILE_WIDTH as f32 / 2.0;
            let screen_y = value.y * TILE_HEIGHT as f32 - (value.x + value.z) * TILE_HEIGHT as f32 / 2.0;

            Vec2::new(screen_x, screen_y).into()
        }
    }
    impl From<&WorldCoords> for ScreenCoords {
        fn from(value: &WorldCoords) -> Self {
            let screen_x = (value.x - value.z) * TILE_WIDTH as f32 / 2.0;
            let screen_y = value.y * TILE_HEIGHT as f32 - (value.x + value.z) * TILE_HEIGHT as f32 / 2.0;

            Vec2::new(screen_x, screen_y).into()
        }
    }
    impl From<TileCoords> for ScreenCoords {
        fn from(value: TileCoords) -> Self {
            let screen_x = (value.x - value.z) * TILE_WIDTH / 2;
            let screen_y = value.y * TILE_HEIGHT - (value.x + value.z) * TILE_HEIGHT / 2;

            Vec2::new(screen_x as f32, screen_y as f32).into()
        }
    }
    impl From<&TileCoords> for ScreenCoords {
        fn from(value: &TileCoords) -> Self {
            let screen_x = (value.x - value.z) * TILE_WIDTH / 2;
            let screen_y = value.y * TILE_HEIGHT - (value.x + value.z) * TILE_HEIGHT / 2;

            Vec2::new(screen_x as f32, screen_y as f32).into()
        }
    }
    impl From<Vec2> for ScreenCoords  {
        fn from(value: Vec2) -> Self {
            ScreenCoords(value)
        }
    }
    impl Deref for ScreenCoords {
        type Target = Vec2;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}