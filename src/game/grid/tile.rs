use crate::ReflectResource;
use crate::game::grid::coords::{ScreenCoords, TileCoords, TilePosition, WorldCoords};
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::asset::{Asset, AssetServer, Assets, Handle};
use bevy::image::{Image, TextureAtlas, TextureAtlasLayout};
use bevy::math::{UVec2, Vec3};
use bevy::prelude::{
    Bundle, ChildSpawner, Children, Component, FromWorld, Reflect, Resource, SpawnRelated,
    SpawnWith, Sprite, Transform, World,
};
use std::ops::{Add, AddAssign};

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
    pub fn new(pos_x: bool, neg_x: bool, pos_z: bool, neg_z: bool) -> Self {
        Self {
            pos_x,
            neg_x,
            pos_z,
            neg_z,
        }
    }

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

    pub fn all() -> Self {
        Self {
            pos_x: true,
            neg_x: true,
            pos_z: true,
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

#[derive(Clone, Debug)]
pub enum TileFacing {
    PosX,
    NegX,
    PosZ,
    NegZ,
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
            TileType::SlopeLower { facing, .. } => match facing {
                TileFacing::PosX => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(1.0, 0.5, 0.0),
                        Vec3::new(1.0, 0.5, 1.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
                TileFacing::NegX => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.5, 0.0),
                        Vec3::new(0.0, 0.5, 1.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
                TileFacing::PosZ => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.5, 1.0),
                        Vec3::new(1.0, 0.5, 1.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
                TileFacing::NegZ => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.5, 0.0),
                        Vec3::new(1.0, 0.5, 0.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
            },
            TileType::SlopeUpper { facing, .. } => match facing {
                TileFacing::PosX => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.5, 0.0),
                        Vec3::new(1.0, 1.0, 0.0),
                        Vec3::new(1.0, 1.0, 1.0),
                        Vec3::new(0.0, 0.5, 1.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
                TileFacing::NegX => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 1.0, 0.0),
                        Vec3::new(0.0, 1.0, 1.0),
                        Vec3::new(1.0, 0.5, 1.0),
                        Vec3::new(1.0, 0.5, 0.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
                TileFacing::PosZ => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.5, 0.0),
                        Vec3::new(0.0, 1.0, 1.0),
                        Vec3::new(1.0, 1.0, 1.0),
                        Vec3::new(1.0, 0.5, 0.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
                TileFacing::NegZ => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 1.0, 0.0),
                        Vec3::new(1.0, 1.0, 0.0),
                        Vec3::new(1.0, 0.5, 1.0),
                        Vec3::new(0.0, 0.5, 1.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
            },
            TileType::Stairs(facing) => match facing {
                TileFacing::PosX => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(1.0, 1.0, 0.0),
                        Vec3::new(1.0, 1.0, 1.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
                TileFacing::NegX => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 1.0, 0.0),
                        Vec3::new(0.0, 1.0, 1.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
                TileFacing::PosZ => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 1.0, 1.0),
                        Vec3::new(1.0, 1.0, 1.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
                TileFacing::NegZ => Collider::convex_hull(
                    vec![
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 1.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 1.0, 0.0),
                        Vec3::new(1.0, 1.0, 0.0),
                    ],
                    *position - Vec3::splat(0.5),
                ),
            },
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
