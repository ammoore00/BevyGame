use crate::game::level::grid::coords::{ScreenCoords, TileCoords, TilePosition, WorldCoords};
use crate::game::level::grid::tile::assets::TileAssets;
use crate::game::level::grid::tile::tile_types::TileType;
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::asset::Assets;
use bevy::image::{TextureAtlas, TextureAtlasLayout};
use bevy::math::{UVec2, Vec3};
use bevy::prelude::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, AddAssign};

pub mod assets;
mod collision;

pub fn plugin(app: &mut App) {
    app.add_plugins(assets::plugin);
}

pub const TILE_WIDTH: i32 = 32;
pub const TILE_HEIGHT: i32 = 16;

pub fn tile(
    tile_type: TileType,
    tile_coords: impl Into<TileCoords> + Clone,
    tile_assets: &TileAssets,
) -> impl Bundle {
    let sprite_sheet = tile_assets.get_asset_set_for_material(tile_type.material);
    let layout = tile_assets.layout().clone();

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
                layout: layout.clone(),
                index: tile_type.index,
            },
        ),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            edge_indices.into_iter().for_each(|index| {
                parent.spawn((
                    Sprite::from_atlas_image(
                        sprite_sheet.clone(),
                        TextureAtlas {
                            layout: layout.clone(),
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

pub mod tile_types;
