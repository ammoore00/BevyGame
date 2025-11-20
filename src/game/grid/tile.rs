use crate::asset_tracking::LoadResource;
use crate::game::grid::coords::{ScreenCoords, TileCoords, TilePosition, WorldCoords};
use crate::game::grid::tile::assets::{TileAssets, TileMaterial};
use crate::game::grid::tile::tile_types::TileType;
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::asset::{Asset, AssetServer, Assets, Handle};
use bevy::image::{Image, TextureAtlas, TextureAtlasLayout};
use bevy::math::{UVec2, Vec3};
use bevy::prelude::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, AddAssign};

pub fn plugin(app: &mut App) {
    app.add_plugins(assets::plugin);
}

pub const TILE_WIDTH: i32 = 32;
pub const TILE_HEIGHT: i32 = 16;

pub fn tile(
    tile_type: TileType,
    tile_coords: impl Into<TileCoords> + Clone,
    tile_assets: &TileAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let sprite_sheet = tile_assets.get_asset_set_for_material(tile_type.material);
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 8, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let edge_indices = Vec::new();

    let world_coords: Vec3 = tile_coords.clone().into().as_vec3();

    (
        Tile,
        TilePosition(tile_coords.clone().into()),
        Transform::from_translation(*Into::<ScreenCoords>::into(tile_coords.into())),
        // Physics
        tile_type.shape.get_collision(world_coords),
        PhysicsData::Static,
        // Rendering
        Sprite::from_atlas_image(
            sprite_sheet.clone(),
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: tile_type.index,
            },
        ),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            edge_indices.into_iter().for_each(|index| {
                parent.spawn((
                    Sprite::from_atlas_image(
                        sprite_sheet.clone(),
                        TextureAtlas {
                            layout: texture_atlas_layout.clone(),
                            index,
                        },
                    ),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.01)),
                ));
            })
        })),
    )
}

#[derive(Component)]
struct Tile;

#[derive(Clone, Debug, Default)]
pub struct TileEdges {
    pub pos_x: bool,
    pub neg_x: bool,
    pub pos_z: bool,
    pub neg_z: bool,
}

impl TileEdges {
    pub fn pos_x() -> Self {
        Self {
            pos_x: true,
            neg_x: false,
            pos_z: false,
            neg_z: false,
        }
    }

    pub fn neg_x() -> Self {
        Self {
            pos_x: false,
            neg_x: true,
            pos_z: false,
            neg_z: false,
        }
    }

    pub fn pos_z() -> Self {
        Self {
            pos_x: false,
            neg_x: false,
            pos_z: true,
            neg_z: false,
        }
    }

    pub fn neg_z() -> Self {
        Self {
            pos_x: false,
            neg_x: false,
            pos_z: false,
            neg_z: true,
        }
    }
}

impl Add for TileEdges {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            pos_x: self.pos_x || other.pos_x,
            neg_x: self.neg_x || other.neg_x,
            pos_z: self.pos_z || other.pos_z,
            neg_z: self.neg_z || other.neg_z,
        }
    }
}

impl AddAssign for TileEdges {
    fn add_assign(&mut self, other: Self) {
        self.pos_x |= other.pos_x;
        self.neg_x |= other.neg_x;
        self.pos_z |= other.pos_z;
        self.neg_z |= other.neg_z;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileFacing {
    PosX,
    NegX,
    PosZ,
    NegZ,
}

impl TileFacing {
    /// Returns the rotation angle in radians around the Y axis for this facing direction
    pub fn rotation_y(&self) -> f32 {
        match self {
            TileFacing::PosX => 0.0,
            TileFacing::PosZ => std::f32::consts::FRAC_PI_2, // 90 degrees
            TileFacing::NegX => std::f32::consts::PI,        // 180 degrees
            TileFacing::NegZ => -std::f32::consts::FRAC_PI_2, // -90 degrees (or 270)
        }
    }

    /// Rotates a point around the Y axis according to this facing direction
    pub fn rotate_point(&self, point: Vec3) -> Vec3 {
        let angle = self.rotation_y();
        let cos = angle.cos();
        let sin = angle.sin();

        Vec3::new(
            point.x * cos - point.z * sin,
            point.y,
            point.x * sin + point.z * cos,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TileShape {
    Full {
        is_top: bool,
    },
    Layer {
        is_top: bool,
    },
    SlopeLower(TileFacing),
    SlopeUpper(TileFacing),
    Stairs(TileFacing),
    Bridge(Option<TileFacing>),
    _Unique {
        id: &'static str,
        provider: fn(WorldCoords) -> Collider,
    },
}

impl PartialEq for TileShape {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TileShape::Full { is_top: a }, TileShape::Full { is_top: b }) => a == b,
            (TileShape::Layer { is_top: a }, TileShape::Layer { is_top: b }) => a == b,
            (TileShape::SlopeLower(a), TileShape::SlopeLower(b)) => a == b,
            (TileShape::SlopeUpper(a), TileShape::SlopeUpper(b)) => a == b,
            (TileShape::Stairs(a), TileShape::Stairs(b)) => a == b,
            (TileShape::Bridge(a), TileShape::Bridge(b)) => a == b,
            (
                TileShape::_Unique {
                    id: a_id,
                    provider: a_provider,
                },
                TileShape::_Unique {
                    id: b_id,
                    provider: b_provider,
                },
            ) => {
                // Function pointers cannot be meaningfully compared,
                // so we instead execute the functions and compare the results
                a_id == b_id && a_provider(Vec3::ZERO.into()) == b_provider(Vec3::ZERO.into())
            }
            _ => false,
        }
    }
}

impl Default for TileShape {
    fn default() -> Self {
        Self::Full { is_top: true }
    }
}

impl TileShape {
    fn get_collision(&self, position: impl Into<WorldCoords>) -> Collider {
        let position = position.into();

        match self {
            TileShape::SlopeLower(facing) => collision::slope(0.0, 0.5, *facing)(position),
            TileShape::SlopeUpper(facing) => collision::slope(0.5, 1.0, *facing)(position),
            TileShape::Stairs(facing) => collision::slope_45(*facing)(position),
            TileShape::Bridge(_) => {
                collision::cuboid(Vec3::new(0.5, 0.25, 0.5))((*position + Vec3::Y * 0.25).into())
            }
            TileShape::_Unique { provider, .. } => provider(position),
            _ => collision::full()(position),
        }
    }
}

mod collision {
    use super::*;

    pub(super) fn full() -> impl Fn(WorldCoords) -> Collider {
        cuboid(Vec3::splat(0.5))
    }

    pub(super) fn slope_45(facing: TileFacing) -> impl Fn(WorldCoords) -> Collider {
        slope(0.0, 1.0, facing)
    }

    pub(super) fn cuboid(size: Vec3) -> impl Fn(WorldCoords) -> Collider {
        move |pos| Collider::cuboid(size, pos)
    }

    pub(super) fn slope(
        lower_height: f32,
        upper_height: f32,
        facing: TileFacing,
    ) -> impl Fn(WorldCoords) -> Collider {
        move |pos| {
            let points = [
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, lower_height, 0.0],
                [0.0, lower_height, 1.0],
                [1.0, upper_height, 1.0],
                [1.0, upper_height, 0.0],
            ]
            .iter()
            .map(|point| facing.rotate_point(Vec3::from(*point) - Vec3::splat(0.5)))
            .collect::<Vec<_>>();

            Collider::convex_hull(points, *pos)
        }
    }
}

pub mod assets {
    use super::*;

    pub(super) fn plugin(app: &mut App) {
        app.load_resource::<TileAssets>();
    }

    #[derive(Resource, Asset, Clone, Reflect)]
    #[reflect(Resource)]
    pub struct TileAssets {
        #[dependency]
        grass: Handle<Image>,
        #[dependency]
        dark_planks: Handle<Image>,
        #[dependency]
        light_planks: Handle<Image>,
        #[dependency]
        dark_framed_planks: Handle<Image>,
        #[dependency]
        light_framed_planks: Handle<Image>,
    }

    impl TileAssets {
        pub(crate) fn get_asset_set_for_material(&self, material: TileMaterial) -> Handle<Image> {
            match material {
                TileMaterial::Grass => self.grass.clone(),
                TileMaterial::DarkPlanks => self.dark_planks.clone(),
                TileMaterial::LightPlanks => self.light_planks.clone(),
                TileMaterial::DarkFramedPlanks => self.dark_framed_planks.clone(),
                TileMaterial::LightFramedPlanks => self.light_framed_planks.clone(),
            }
        }
    }

    impl FromWorld for TileAssets {
        fn from_world(world: &mut World) -> Self {
            let assets = world.resource::<AssetServer>();
            TileAssets {
                grass: assets.load("images/grass.png"),
                dark_planks: assets.load("images/planks.png"),
                light_planks: assets.load("images/light_planks.png"),
                dark_framed_planks: assets.load("images/framed_planks.png"),
                light_framed_planks: assets.load("images/light_framed_planks.png"),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
    pub enum TileMaterial {
        Grass,
        DarkPlanks,
        LightPlanks,
        DarkFramedPlanks,
        LightFramedPlanks,
    }
}

pub mod tile_types {
    use super::*;
    use std::sync::LazyLock;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct TileType {
        pub shape: TileShape,
        pub material: TileMaterial,
        pub index: usize,
    }

    static ALL_TILES: LazyLock<Vec<TileType>> = LazyLock::new(|| {
        let mut vec = Vec::new();
        vec.extend_from_slice(grass::STANDARD_SET);
        vec.extend_from_slice(dark_planks::STANDARD_SET);
        vec.extend_from_slice(dark_framed_planks::STANDARD_SET);
        vec.extend_from_slice(light_planks::STANDARD_SET);
        vec.extend_from_slice(light_framed_planks::STANDARD_SET);
        vec
    });

    impl TileType {
        const fn new(shape: TileShape, material: TileMaterial, index: usize) -> Self {
            Self {
                shape,
                material,
                index,
            }
        }

        pub fn get_tile(shape: TileShape, material: TileMaterial) -> Self {
            *ALL_TILES
                .iter()
                .find(|tile| tile.shape == shape && tile.material == material)
                .unwrap_or_else(|| {
                    panic!(
                        "No tile with the given shape: {shape:?} and material: {material:?} exists"
                    )
                })
        }
    }

    const FULL_INDICES: &[usize] = &[0, 1];
    const LAYER_INDICES: &[usize] = &[2, 3];

    const SLOPE_LOWER_INDICES: &[usize] = &[8, 9, 10, 11];
    const SLOPE_UPPER_INDICES: &[usize] = &[12, 13, 14, 15];

    const STAIRS_INDICES: &[usize] = &[24, 25, 26, 27];

    const BRIDGE_INDICES: &[usize] = &[32, 33, 34, 35, 36];

    macro_rules! standard_tile_set {
        ($material:ident) => {
            pub const FULL: TileType =
                TileType::new(TileShape::Full { is_top: true }, $material, FULL_INDICES[0]);
            pub const FULL_BOTTOM: TileType = TileType::new(
                TileShape::Full { is_top: false },
                $material,
                FULL_INDICES[1],
            );

            pub const LAYER: TileType = TileType::new(
                TileShape::Layer { is_top: true },
                $material,
                LAYER_INDICES[0],
            );
            pub const LAYER_BOTTOM: TileType = TileType::new(
                TileShape::Layer { is_top: false },
                $material,
                LAYER_INDICES[1],
            );

            pub const SLOPE_LOWER_NEG_X: TileType = TileType::new(
                TileShape::SlopeLower(TileFacing::NegX),
                $material,
                SLOPE_LOWER_INDICES[0],
            );
            pub const SLOPE_LOWER_NEG_Z: TileType = TileType::new(
                TileShape::SlopeLower(TileFacing::NegZ),
                $material,
                SLOPE_LOWER_INDICES[1],
            );
            pub const SLOPE_LOWER_POS_X: TileType = TileType::new(
                TileShape::SlopeLower(TileFacing::PosX),
                $material,
                SLOPE_LOWER_INDICES[2],
            );
            pub const SLOPE_LOWER_POS_Z: TileType = TileType::new(
                TileShape::SlopeLower(TileFacing::PosZ),
                $material,
                SLOPE_LOWER_INDICES[3],
            );

            pub const SLOPE_UPPER_NEG_X: TileType = TileType::new(
                TileShape::SlopeUpper(TileFacing::NegX),
                $material,
                SLOPE_UPPER_INDICES[0],
            );
            pub const SLOPE_UPPER_NEG_Z: TileType = TileType::new(
                TileShape::SlopeUpper(TileFacing::NegZ),
                $material,
                SLOPE_UPPER_INDICES[1],
            );
            pub const SLOPE_UPPER_POS_X: TileType = TileType::new(
                TileShape::SlopeUpper(TileFacing::PosX),
                $material,
                SLOPE_UPPER_INDICES[2],
            );
            pub const SLOPE_UPPER_POS_Z: TileType = TileType::new(
                TileShape::SlopeUpper(TileFacing::PosZ),
                $material,
                SLOPE_UPPER_INDICES[3],
            );

            pub const STAIRS_NEG_X: TileType = TileType::new(
                TileShape::Stairs(TileFacing::NegX),
                $material,
                STAIRS_INDICES[0],
            );
            pub const STAIRS_NEG_Z: TileType = TileType::new(
                TileShape::Stairs(TileFacing::NegZ),
                $material,
                STAIRS_INDICES[1],
            );
            pub const STAIRS_POS_X: TileType = TileType::new(
                TileShape::Stairs(TileFacing::PosX),
                $material,
                STAIRS_INDICES[2],
            );
            pub const STAIRS_POS_Z: TileType = TileType::new(
                TileShape::Stairs(TileFacing::PosZ),
                $material,
                STAIRS_INDICES[3],
            );

            pub const BRIDGE: TileType =
                TileType::new(TileShape::Bridge(None), $material, BRIDGE_INDICES[0]);
            pub const BRIDGE_NEG_X: TileType = TileType::new(
                TileShape::Bridge(Some(TileFacing::NegX)),
                $material,
                BRIDGE_INDICES[1],
            );
            pub const BRIDGE_NEG_Z: TileType = TileType::new(
                TileShape::Bridge(Some(TileFacing::NegZ)),
                $material,
                BRIDGE_INDICES[2],
            );
            pub const BRIDGE_POS_X: TileType = TileType::new(
                TileShape::Bridge(Some(TileFacing::PosX)),
                $material,
                BRIDGE_INDICES[3],
            );
            pub const BRIDGE_POS_Z: TileType = TileType::new(
                TileShape::Bridge(Some(TileFacing::PosZ)),
                $material,
                BRIDGE_INDICES[4],
            );

            pub const STANDARD_SET: &[TileType] = &[
                FULL,
                FULL_BOTTOM,
                LAYER,
                LAYER_BOTTOM,
                SLOPE_LOWER_POS_X,
                SLOPE_LOWER_POS_Z,
                SLOPE_LOWER_NEG_X,
                SLOPE_LOWER_NEG_Z,
                SLOPE_UPPER_POS_X,
                SLOPE_UPPER_POS_Z,
                SLOPE_UPPER_NEG_X,
                SLOPE_UPPER_NEG_Z,
                STAIRS_POS_X,
                STAIRS_POS_Z,
                STAIRS_NEG_X,
                STAIRS_NEG_Z,
                BRIDGE,
                BRIDGE_POS_X,
                BRIDGE_POS_Z,
                BRIDGE_NEG_X,
                BRIDGE_NEG_Z,
            ];
        };
    }

    pub mod grass {
        use super::*;

        const MATERIAL: TileMaterial = TileMaterial::Grass;
        standard_tile_set!(MATERIAL);
    }

    pub mod dark_planks {
        use super::*;

        const MATERIAL: TileMaterial = TileMaterial::DarkPlanks;
        standard_tile_set!(MATERIAL);
    }

    pub mod dark_framed_planks {
        use super::*;

        const MATERIAL: TileMaterial = TileMaterial::DarkFramedPlanks;
        standard_tile_set!(MATERIAL);
    }

    pub mod light_planks {
        use super::*;

        const MATERIAL: TileMaterial = TileMaterial::LightPlanks;
        standard_tile_set!(MATERIAL);
    }

    pub mod light_framed_planks {
        use super::*;

        const MATERIAL: TileMaterial = TileMaterial::LightFramedPlanks;
        standard_tile_set!(MATERIAL);
    }
}
