use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use crate::ReflectResource;
use bevy::prelude::*;
use crate::asset_tracking::LoadResource;
use crate::game::grid::coords::{ScreenCoords, TileCoords, TilePosition};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TileDebugAssets>();

    app.add_plugins(coords::plugin);
}

#[derive(Component)]
pub struct Grid(pub Arc<RwLock<BTreeMap<TileCoords, Entity>>>);

pub fn grid(
    tile_debug_assets: TileDebugAssets,
    scale: f32,
) -> impl Bundle {
    let tile_map = Arc::new(RwLock::new(BTreeMap::<TileCoords, Entity>::new()));

    let tile_coords = vec![
        IVec3::new(0, 0, 0),
        IVec3::new(0, 0, 1),
        IVec3::new(0, 0, 2),

        IVec3::new(1, 0, 2),
        IVec3::new(2, 0, 2),

        IVec3::new(2, 0, 1),
        IVec3::new(2, 0, 0),
        IVec3::new(2, 0, -1),
        IVec3::new(2, 0, -2),

        IVec3::new(1, 0, -2),
        IVec3::new(0, 0, -2),
        IVec3::new(-1, 0, -2),
        IVec3::new(-2, 0, -2),

        IVec3::new(-2, 0, -1),
        IVec3::new(-2, 0, 0),
        IVec3::new(-2, 0, 1),
        IVec3::new(-2, 0, 2),
    ];

    (
        Grid(tile_map.clone()),
        Transform::from_scale(Vec2::splat(scale).extend(1.0)),
        InheritedVisibility::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            for coord in tile_coords {
                let tile = parent.spawn((
                    tile(
                        coord,
                        &tile_debug_assets
                    ),
                )).id();

                tile_map.write().unwrap().insert(coord.into(), tile);
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
    use std::cmp::Ordering;
    use std::ops::Deref;
    use bevy::prelude::*;
    use crate::game::grid::{TILE_HEIGHT, TILE_WIDTH};
    use crate::Scale;

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
        scale: Res<Scale>,
        mut query: Query<(&WorldPosition, &mut Transform), Changed<WorldPosition>>,
    ) {
        for (world_position, mut transform) in query.iter_mut() {
            let scaled_coords: WorldCoords = (*world_position.0 * scale.0).into();

            let screen_coords = ScreenCoords::from(scaled_coords);
            transform.translation = screen_coords.extend(0.0);
        }
    }

    #[derive(Component, Debug)]
    pub struct TilePosition(pub TileCoords);
    pub fn convert_tile_to_screen_coords(
        scale: Res<Scale>,
        mut query: Query<(&TilePosition, &mut Transform), Changed<TilePosition>>,
    ) {
        for (tile_position, mut transform) in query.iter_mut() {
            let scaled_coords: WorldCoords = (tile_position.0.as_vec3() * scale.0).into();

            let screen_coords = ScreenCoords::from(&tile_position.0);
            transform.translation = screen_coords.extend(0.0);
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Reflect)]
    pub struct TileCoords(pub IVec3);
    impl From<WorldCoords> for TileCoords {
        fn from(value: WorldCoords) -> Self {
            Self::from(value.0)
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
    impl From<Vec3> for TileCoords  {
        fn from(value: Vec3) -> Self {
            // Use round() instead of as_ivec3() to get proper rounding
            TileCoords(IVec3::new(
                value.x.round() as i32,
                value.y.round() as i32,
                value.z.round() as i32,
            ))
        }
    }
    impl Deref for TileCoords {
        type Target = IVec3;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Ord for TileCoords {
        fn cmp(&self, other: &Self) -> Ordering {
            match self.y.cmp(&other.y) {
                Ordering::Equal => match self.z.cmp(&other.z) {
                    Ordering::Equal => self.x.cmp(&other.x),
                    ordering => ordering,
                },
                ordering => ordering,
            }
        }
    }
    impl PartialOrd for TileCoords {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    
    #[derive(Debug)]
    pub struct WorldCoords(pub Vec3);
    impl From<TileCoords> for WorldCoords {
        fn from(value: TileCoords) -> Self {
            Self::from(value.0)
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
    impl From<IVec3> for WorldCoords  {
        fn from(value: IVec3) -> Self {
            WorldCoords(value.as_vec3())
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

    pub fn rotate_movement_to_screen_space(movement: Vec3) -> Vec3 {
        let angle = -std::f32::consts::FRAC_PI_4;
        let rotation = Quat::from_rotation_y(angle);
        rotation * movement
    }

    pub fn rotate_screen_space_to_movement(screen_space: Vec3) -> Vec3 {
        let angle = std::f32::consts::FRAC_PI_4;
        let rotation = Quat::from_rotation_y(angle);
        rotation * screen_space
    }
}