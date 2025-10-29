use crate::ReflectResource;
use crate::asset_tracking::LoadResource;
use crate::game::grid::coords::{SCREEN_Z_SCALE, ScreenCoords, TileCoords, TilePosition};
use bevy::prelude::*;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TileAssets>();

    app.add_plugins(coords::plugin);
}

#[derive(Component)]
pub struct Grid(pub Arc<RwLock<BTreeMap<TileCoords, Entity>>>);

pub fn grid(
    tile_map: Arc<RwLock<BTreeMap<TileCoords, Entity>>>,
    tile_assets: TileAssets,
    scale: f32,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    (
        Grid(tile_map.clone()),
        Transform::from_scale(Vec2::splat(scale).extend(SCREEN_Z_SCALE)),
        InheritedVisibility::default()
    )
}

pub const TILE_WIDTH: i32 = 32;
pub const TILE_HEIGHT: i32 = 16;

#[derive(Component)]
struct Tile;

#[derive(Clone, Debug)]
pub enum TileType {
    Full,
    Layer,
    Stairs(TileFacing),
    Bridge(Option<TileFacing>),
}

impl TileType {
    fn get_collision(&self) -> TileCollision {
        match self {
            TileType::Stairs(facing) => match facing {
                TileFacing::PosX => TileCollision::new(1.0, 1.0, 0.0, 0.0),
                TileFacing::NegX => TileCollision::new(0.0, 0.0, 1.0, 1.0),
                TileFacing::PosZ => TileCollision::new(1.0, 0.0, 1.0, 0.0),
                TileFacing::NegZ => TileCollision::new(0.0, 1.0, 0.0, 1.0),
            },
            _ => TileCollision::default(),
        }
    }

    fn get_atlas_index(&self) -> usize {
        match self {
            TileType::Full => 0,
            TileType::Layer => 1,
            TileType::Stairs(facing) => {
                match facing {
                    TileFacing::NegX => 16,
                    TileFacing::NegZ => 17,
                    TileFacing::PosX => 18,
                    TileFacing::PosZ => 19,
                }
            },
            TileType::Bridge(facing) => {
                match facing {
                    Some(facing) => {
                        match facing {
                            TileFacing::NegX => 25,
                            TileFacing::NegZ => 26,
                            TileFacing::PosX => 27,
                            TileFacing::PosZ => 28,
                        }
                    }
                    None => 24,
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum TileFacing {
    PosX,
    NegX,
    PosZ,
    NegZ,
}

#[derive(Clone, Debug, Component)]
pub struct TileCollision {
    pub pp: f32,
    pub pn: f32,
    pub np: f32,
    pub nn: f32,
}

impl TileCollision {
    fn level(height: f32) -> Self {
        Self::new(height, height, height, height)
    }

    fn new(pp: f32, pn: f32, np: f32, nn: f32) -> Self {
        Self { pp, pn, np, nn }
    }

    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        //println!("x: {}, z: {}", x, z);

        let mut frac_x = (x + 0.5).fract();
        let mut frac_z = (z + 0.5).fract();

        if frac_x < 0.0 {
            frac_x += 1.0;
        }

        if frac_z < 0.0 {
            frac_z += 1.0;
        }

        //println!("frac_x: {}, frac_z: {}", frac_x, frac_z);

        self.bilerp(frac_x, frac_z).clamp(0.0, 1.0)
    }

    fn bilerp(&self, x: f32, z: f32) -> f32 {
        // Bilinear interpolation between four corners
        let x = x.clamp(0.0, 1.0);
        let z = z.clamp(0.0, 1.0);

        let x1 = self.nn + x * (self.pn - self.nn);
        let x2 = self.np + x * (self.pp - self.np);

        x1 + z * (x2 - x1)
    }
}

impl Default for TileCollision {
    fn default() -> Self {
        Self::level(1.0)
    }
}

pub fn tile(
    tile_type: TileType,
    tile_material: TileMaterial,
    tile_coords: impl Into<TileCoords> + Clone,
    tile_assets: &TileAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let sprite_sheet = tile_assets.get_asset_set_for_material(&tile_material);
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 8, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    (
        Tile,
        TilePosition(tile_coords.clone().into()),
        tile_type.get_collision(),
        Sprite::from_atlas_image(
            sprite_sheet.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: tile_type.get_atlas_index(),
            },
        ),
        Transform::from_translation(*Into::<ScreenCoords>::into(tile_coords.into())),
    )
}

#[derive(Clone, Debug)]
pub enum TileMaterial {
    Grass,
    Stone,
    Planks,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct TileAssets {
    #[dependency]
    grass: Handle<Image>,
    #[dependency]
    stone: Handle<Image>,
    #[dependency]
    planks: Handle<Image>,
}

impl TileAssets {
    fn get_asset_set_for_material(&self, material: &TileMaterial) -> Handle<Image> {
        match material {
            TileMaterial::Grass => self.grass.clone(),
            TileMaterial::Stone => self.stone.clone(),
            TileMaterial::Planks => self.planks.clone(),
        }
    }
}

impl FromWorld for TileAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        TileAssets {
            grass: assets.load("images/grass.png"),
            stone: assets.load("images/stone.png"),
            planks: assets.load("images/planks.png"),
        }
    }
}

pub mod coords {
    use crate::Scale;
    use crate::game::grid::{TILE_HEIGHT, TILE_WIDTH};
    use bevy::prelude::*;
    use std::cmp::Ordering;
    use std::ops::Deref;

    pub(super) const SCREEN_Z_SCALE: f32 = 2.0;

    pub(super) fn plugin(app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                convert_world_to_screen_coords,
                convert_tile_to_screen_coords,
            ),
        );
    }

    #[derive(Component, Debug)]
    pub struct WorldPosition(pub WorldCoords);
    pub fn convert_world_to_screen_coords(
        scale: Res<Scale>,
        mut query: Query<(&WorldPosition, &mut Transform), Changed<WorldPosition>>,
    ) {
        for (world_position, mut transform) in query.iter_mut() {
            let screen_coords = ScreenCoords::from(&world_position.0);
            transform.translation = *screen_coords;
            transform.translation.x *= scale.0;
            transform.translation.y *= scale.0;
            // Offset to render in front of tiles
            transform.translation.z += SCREEN_Z_SCALE;
        }
    }

    #[derive(Component, Debug)]
    pub struct TilePosition(pub TileCoords);
    pub fn convert_tile_to_screen_coords(
        mut query: Query<(&TilePosition, &mut Transform), Changed<TilePosition>>,
    ) {
        for (tile_position, mut transform) in query.iter_mut() {
            let screen_coords = ScreenCoords::from(&tile_position.0);
            transform.translation = *screen_coords;

            transform.translation.y -= TILE_HEIGHT as f32;
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
    impl From<IVec3> for TileCoords {
        fn from(value: IVec3) -> Self {
            TileCoords(value)
        }
    }
    impl From<Vec3> for TileCoords {
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
    impl From<Vec3> for WorldCoords {
        fn from(value: Vec3) -> Self {
            WorldCoords(value)
        }
    }
    impl From<IVec3> for WorldCoords {
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
    pub struct ScreenCoords(pub Vec3);
    impl From<WorldCoords> for ScreenCoords {
        fn from(value: WorldCoords) -> Self {
            let screen_x = (value.x - value.z) * TILE_WIDTH as f32 / 2.0;
            let screen_y =
                (value.y * TILE_HEIGHT as f32) - (value.x + value.z) * TILE_HEIGHT as f32 / 2.0;

            let screen_z = (value.x + value.z + value.y) * SCREEN_Z_SCALE;

            Vec3::new(screen_x, screen_y, screen_z).into()
        }
    }
    impl From<&WorldCoords> for ScreenCoords {
        fn from(value: &WorldCoords) -> Self {
            let screen_x = (value.x - value.z) * TILE_WIDTH as f32 / 2.0;
            let screen_y =
                (value.y * TILE_HEIGHT as f32) - (value.x + value.z) * TILE_HEIGHT as f32 / 2.0;

            let screen_z = (value.x + value.z + value.y) * SCREEN_Z_SCALE;

            Vec3::new(screen_x, screen_y, screen_z).into()
        }
    }
    impl From<TileCoords> for ScreenCoords {
        fn from(value: TileCoords) -> Self {
            let screen_x = (value.x - value.z) * TILE_WIDTH / 2;
            let screen_y = (value.y * TILE_HEIGHT) - (value.x + value.z) * TILE_HEIGHT / 2;

            let screen_z = value.x as f32 + value.z as f32 + value.y as f32;

            Vec3::new(screen_x as f32, screen_y as f32, screen_z).into()
        }
    }
    impl From<&TileCoords> for ScreenCoords {
        fn from(value: &TileCoords) -> Self {
            let screen_x = (value.x - value.z) * TILE_WIDTH / 2;
            let screen_y = (value.y * TILE_HEIGHT) - (value.x + value.z) * TILE_HEIGHT / 2;

            let screen_z = value.x as f32 + value.z as f32 + value.y as f32;

            Vec3::new(screen_x as f32, screen_y as f32, screen_z).into()
        }
    }
    impl From<Vec3> for ScreenCoords {
        fn from(value: Vec3) -> Self {
            ScreenCoords(value)
        }
    }
    impl Deref for ScreenCoords {
        type Target = Vec3;
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
