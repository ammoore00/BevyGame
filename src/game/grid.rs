use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use crate::ReflectResource;
use bevy::prelude::*;
use crate::asset_tracking::LoadResource;
use crate::game::grid::coords::{ScreenCoords, TileCoords, TilePosition, SCREEN_Z_SCALE};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TileAssets>();

    app.add_plugins(coords::plugin);
}

#[derive(Component)]
pub struct Grid(pub Arc<RwLock<BTreeMap<TileCoords, Entity>>>);

pub fn grid(
    tile_assets: TileAssets,
    scale: f32,
) -> impl Bundle {
    let tile_map = Arc::new(RwLock::new(BTreeMap::<TileCoords, Entity>::new()));

    let start_x = -2;
    let start_z = -3;
    let level = [
        "XXXXX..XXXXXX.",
        "XXXXX..XXXXXX.",
        "XXXXX..X....X.",
        "XXXXXXXX.XXXX.",
        "XXXXX....X....",
        "..X......XXXX.",
        "..X.........X.",
        ".XXX.......XXX",
        ".XXXXXXXXXXXXX",
        ".XXX.......XXX",
    ];

    let mut tile_coords = Vec::new();

    let mut z = start_z;

    for &row in level.iter() {
        let mut x = start_x;

        for col in String::from(row).chars() {
            if col == 'X' {
                tile_coords.push((TileMaterial::Grass, TileType::Layer, TileCoords(IVec3::new(x, 0, z))));
            }
            x += 1;
        }
        z += 1;
    }

    tile_coords.push((TileMaterial::Grass, TileType::Full, TileCoords(IVec3::new(-2, 1, -3))));
    tile_coords.push((TileMaterial::Grass, TileType::Full, TileCoords(IVec3::new(-1, 1, -3))));
    tile_coords.push((TileMaterial::Grass, TileType::Full, TileCoords(IVec3::new(0, 1, -3))));
    tile_coords.push((TileMaterial::Grass, TileType::Full, TileCoords(IVec3::new(1, 1, -3))));
    tile_coords.push((TileMaterial::Grass, TileType::Full, TileCoords(IVec3::new(2, 1, -3))));

    tile_coords.push((TileMaterial::Grass, TileType::Full, TileCoords(IVec3::new(-2, 1, -2))));
    tile_coords.push((TileMaterial::Grass, TileType::Full, TileCoords(IVec3::new(-1, 1, -2))));
    tile_coords.push((TileMaterial::Grass, TileType::Stairs(TileFacing::NegX), TileCoords(IVec3::new(0, 1, -2))));

    tile_coords.push((TileMaterial::Grass, TileType::Full, TileCoords(IVec3::new(-1, 1, -1))));
    tile_coords.push((TileMaterial::Grass, TileType::Stairs(TileFacing::NegZ), TileCoords(IVec3::new(-1, 1, 0))));

    tile_coords.push((TileMaterial::Grass, TileType::Stairs(TileFacing::NegZ), TileCoords(IVec3::new(2, 1, -2))));

    tile_coords.push((TileMaterial::Grass, TileType::Full, TileCoords(IVec3::new(-2, 1, -1))));
    tile_coords.push((TileMaterial::Grass, TileType::Full, TileCoords(IVec3::new(-2, 1, 0))));
    tile_coords.push((TileMaterial::Grass, TileType::Full, TileCoords(IVec3::new(-2, 1, 1))));

    tile_coords.push((TileMaterial::Stone, TileType::Full, TileCoords(IVec3::new(0, 1, 5))));

    (
        Grid(tile_map.clone()),
        Transform::from_scale(Vec2::splat(scale).extend(SCREEN_Z_SCALE)),
        InheritedVisibility::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            for (material, tile_type, coords) in tile_coords {
                let tile = parent.spawn((
                    tile(
                        tile_type,
                        material,
                        coords.clone(),
                        &tile_assets
                    ),
                )).id();

                tile_map.write().unwrap().insert(coords.into(), tile);
            }
        })),
    )
}

pub const TILE_WIDTH: i32 = 32;
pub const TILE_HEIGHT: i32 = 16;

#[derive(Component)]
struct Tile {
    tile_type: TileType,
    tile_material: TileMaterial,
}

#[derive(Clone, Debug)]
pub enum TileType {
    Full,
    Half,
    Layer,
    SlopeLower(TileFacing),
    SlopeUpper(TileFacing),
    Stairs(TileFacing),
}

impl TileType {
    fn get_collision(&self) -> TileCollision {
        match self {
            TileType::Half => TileCollision::level(0.5),
            TileType::SlopeLower(_) => todo!(),
            TileType::SlopeUpper(_) => todo!(),
            TileType::Stairs(facing) => {
                match facing {
                    TileFacing::PosX => TileCollision::new(1.0, 1.0, 0.0, 0.0),
                    TileFacing::NegX => TileCollision::new(0.0, 0.0, 1.0, 1.0),
                    TileFacing::PosZ => TileCollision::new(1.0, 0.0, 1.0, 0.0),
                    TileFacing::NegZ => TileCollision::new(0.0, 1.0, 0.0, 1.0),
                }
            }
            _ => TileCollision::default(),
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

#[derive(Clone, Debug)]
pub enum TileMaterial {
    Grass,
    Stone,
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
        Self {
            pp, pn, np, nn,
        }
    }
    
    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        let mut frac_x = x.fract() + 0.5;
        let mut frac_z = z.fract() + 0.5;

        if frac_x < 0.0 {
            frac_x += 1.0;
        }

        if frac_z < 0.0 {
            frac_z += 1.0;
        }

        self.bilerp(frac_x, frac_z)
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
) -> impl Bundle {
    let asset_set = tile_assets.get_asset_set_for_material(&tile_material);
    let sprite = asset_set.get_sprite_for_type(&tile_type);

    (
        Tile {
            tile_type: tile_type.clone(),
            tile_material,
        },
        TilePosition(tile_coords.clone().into()),
        tile_type.get_collision(),
        Sprite::from(sprite),
        Transform::from_translation(*Into::<ScreenCoords>::into(tile_coords.into())),
    )
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct TileAssets {
    #[dependency]
    grass: TileAssetSet,
    #[dependency]
    stone: TileAssetSet,
}

impl TileAssets {
    fn get_asset_set_for_material(&self, material: &TileMaterial) -> &TileAssetSet {
        match material {
            TileMaterial::Grass => &self.grass,
            TileMaterial::Stone => &self.stone,
        }
    }
}

#[derive(Asset, Clone, Reflect)]
struct TileAssetSet {
    full: Handle<Image>,
    half: Handle<Image>,
    layer: Handle<Image>,

    //slope_lower_pos_x: Handle<Image>,
    //slope_lower_neg_x: Handle<Image>,
    //slope_lower_pos_z: Handle<Image>,
    //slope_lower_neg_z: Handle<Image>,

    //slope_upper_pos_x: Handle<Image>,
    //slope_upper_neg_x: Handle<Image>,
    //slope_upper_pos_z: Handle<Image>,
    //slope_upper_neg_z: Handle<Image>,

    stairs_pos_x: Handle<Image>,
    stairs_neg_x: Handle<Image>,
    stairs_pos_z: Handle<Image>,
    stairs_neg_z: Handle<Image>,
}

impl TileAssetSet {
    fn new(name: &str, assets: &AssetServer) -> Self {
        Self {
            full: assets.load(format!{"images/{name}.png"}),
            half: assets.load(format!{"images/{name}_half.png"}),
            layer: assets.load(format!{"images/{name}_layer.png"}),
            stairs_pos_x: assets.load(format!{"images/{name}_stairs_pos_x.png"}),
            stairs_neg_x: assets.load(format!{"images/{name}_stairs_neg_x.png"}),
            stairs_pos_z: assets.load(format!{"images/{name}_stairs_pos_z.png"}),
            stairs_neg_z: assets.load(format!{"images/{name}_stairs_neg_z.png"}),
        }
    }

    fn get_sprite_for_type(&self, tile_type: &TileType) -> Handle<Image> {
        match tile_type {
            TileType::Full => self.full.clone(),
            TileType::Half => self.half.clone(),
            TileType::Layer => self.layer.clone(),
            TileType::SlopeLower(_) => todo!(),
            TileType::SlopeUpper(_) => todo!(),
            TileType::Stairs(facing) => {
                match facing {
                    TileFacing::PosX => self.stairs_pos_x.clone(),
                    TileFacing::NegX => self.stairs_neg_x.clone(),
                    TileFacing::PosZ => self.stairs_pos_z.clone(),
                    TileFacing::NegZ => self.stairs_neg_z.clone(),
                }
            }
        }
    }
}

impl FromWorld for TileAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        TileAssets {
            grass: TileAssetSet::new("grass", assets),
            stone: TileAssetSet::new("stone", assets),
        }
    }
}

pub mod coords {
    use std::cmp::Ordering;
    use std::ops::Deref;
    use bevy::prelude::*;
    use crate::game::grid::{TILE_HEIGHT, TILE_WIDTH};
    use crate::Scale;

    pub(super) const SCREEN_Z_SCALE: f32 = 2.0;

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
    pub struct ScreenCoords(pub Vec3);
    impl From<WorldCoords> for ScreenCoords {
        fn from(value: WorldCoords) -> Self {
            let screen_x = (value.x - value.z) * TILE_WIDTH as f32 / 2.0;
            let screen_y = (value.y * TILE_HEIGHT as f32) - (value.x + value.z) * TILE_HEIGHT as f32 / 2.0;

            let screen_z = (value.x + value.z + value.y) * SCREEN_Z_SCALE;

            Vec3::new(screen_x, screen_y, screen_z).into()
        }
    }
    impl From<&WorldCoords> for ScreenCoords {
        fn from(value: &WorldCoords) -> Self {
            let screen_x = (value.x - value.z) * TILE_WIDTH as f32 / 2.0;
            let screen_y = (value.y * TILE_HEIGHT as f32) - (value.x + value.z) * TILE_HEIGHT as f32 / 2.0;

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
    impl From<Vec3> for ScreenCoords  {
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