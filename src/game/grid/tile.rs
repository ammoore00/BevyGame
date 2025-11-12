use crate::ReflectResource;
use crate::asset_tracking::LoadResource;
use crate::game::grid::coords::{ScreenCoords, TileCoords, TilePosition, WorldCoords};
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::asset::{Asset, AssetServer, Assets, Handle};
use bevy::image::{Image, TextureAtlas, TextureAtlasLayout};
use bevy::math::{UVec2, Vec3};
use bevy::prelude::*;
use std::ops::{Add, AddAssign};

pub fn plugin(app: &mut App) {
    app.load_resource::<TileAssets>();
}

pub const TILE_WIDTH: i32 = 32;
pub const TILE_HEIGHT: i32 = 16;

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

    let edge_indices = tile_type.get_edge_overlay_atlas_indices();

    let world_coords: Vec3 = tile_coords.clone().into().as_vec3();

    (
        Tile,
        TilePosition(tile_coords.clone().into()),
        Transform::from_translation(*Into::<ScreenCoords>::into(tile_coords.into())),
        // Physics
        tile_type.get_collision(world_coords),
        PhysicsData::Static,
        // Rendering
        Sprite::from_atlas_image(
            sprite_sheet.clone(),
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: tile_type.get_atlas_index(),
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

#[derive(Clone, Debug)]
pub enum TileType {
    Full {
        is_top: bool,
        edges: TileEdges,
    },
    Layer {
        is_top: bool,
        edges: TileEdges,
    },
    SlopeLower {
        facing: TileFacing,
        has_edge: bool,
    },
    SlopeUpper {
        facing: TileFacing,
        has_edge: bool,
    },
    Stairs(TileFacing),
    Bridge {
        facing: Option<TileFacing>,
        edges: TileEdges,
    },
}

impl Default for TileType {
    fn default() -> Self {
        Self::Full {
            is_top: true,
            edges: TileEdges::default(),
        }
    }
}

impl TileType {
    fn get_collision(&self, position: impl Into<WorldCoords>) -> Collider {
        let position = position.into();

        match self {
            TileType::SlopeLower { facing, .. } => collision::slope(0.0, 0.5, *facing)(position),
            TileType::SlopeUpper { facing, .. } => collision::slope(0.5, 1.0, *facing)(position),
            TileType::Stairs(facing) => collision::slope_45(*facing)(position),
            TileType::Bridge { .. } => {
                collision::cuboid(Vec3::new(0.5, 0.25, 0.5))((*position + Vec3::Y * 0.25).into())
            }
            _ => Collider::cuboid(Vec3::splat(0.5), position),
        }
    }

    fn get_atlas_index(&self) -> usize {
        match self {
            TileType::Full { is_top: true, .. } => 0,
            TileType::Full { is_top: false, .. } => 1,
            TileType::Layer { is_top: true, .. } => 2,
            TileType::Layer { is_top: false, .. } => 3,
            TileType::SlopeLower { facing, .. } => match facing {
                TileFacing::NegX => 8,
                TileFacing::NegZ => 9,
                TileFacing::PosX => 10,
                TileFacing::PosZ => 11,
            },
            TileType::SlopeUpper { facing, .. } => match facing {
                TileFacing::NegX => 12,
                TileFacing::NegZ => 13,
                TileFacing::PosX => 14,
                TileFacing::PosZ => 15,
            },
            TileType::Stairs(facing) => match facing {
                TileFacing::NegX => 24,
                TileFacing::NegZ => 25,
                TileFacing::PosX => 26,
                TileFacing::PosZ => 27,
            },
            TileType::Bridge { facing, .. } => match facing {
                Some(facing) => match facing {
                    TileFacing::NegX => 33,
                    TileFacing::NegZ => 34,
                    TileFacing::PosX => 35,
                    TileFacing::PosZ => 36,
                },
                None => 32,
            },
        }
    }

    fn get_edge_overlay_atlas_indices(&self) -> Vec<usize> {
        match self {
            TileType::SlopeLower {
                facing,
                has_edge: true,
            } => vec![match facing {
                TileFacing::NegX => 16,
                TileFacing::NegZ => 17,
                TileFacing::PosX => 18,
                TileFacing::PosZ => 19,
            }],
            TileType::SlopeUpper {
                facing,
                has_edge: true,
            } => vec![match facing {
                TileFacing::NegX => 20,
                TileFacing::NegZ => 21,
                TileFacing::PosX => 22,
                TileFacing::PosZ => 23,
            }],
            TileType::Full {
                edges,
                is_top: true,
            }
            | TileType::Layer {
                edges,
                is_top: true,
            }
            | TileType::Bridge { edges, .. } => {
                let mut indices = Vec::new();

                if edges.pos_x {
                    indices.push(6);
                }
                if edges.pos_z {
                    indices.push(7);
                }

                if edges.neg_x {
                    indices.push(4);
                }
                if edges.neg_z {
                    indices.push(5);
                }

                indices
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TileMaterial {
    Grass,
    Stone,
    Planks,
    FramedPlanks,
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
    #[dependency]
    framed_planks: Handle<Image>,
}

impl TileAssets {
    fn get_asset_set_for_material(&self, material: &TileMaterial) -> Handle<Image> {
        match material {
            TileMaterial::Grass => self.grass.clone(),
            TileMaterial::Stone => self.stone.clone(),
            TileMaterial::Planks => self.planks.clone(),
            TileMaterial::FramedPlanks => self.framed_planks.clone(),
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
            framed_planks: assets.load("images/framed_planks.png"),
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
