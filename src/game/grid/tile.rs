use crate::ReflectResource;
use crate::game::grid::coords::{ScreenCoords, TileCoords, TilePosition};
use bevy::asset::{Asset, AssetServer, Assets, Handle};
use bevy::image::{Image, TextureAtlas, TextureAtlasLayout};
use bevy::math::{UVec2, Vec3};
use bevy::prelude::{Bundle, Component, FromWorld, Reflect, Resource, Sprite, Transform, World};
use crate::game::physics::components::{Collider, PhysicsData};

pub const TILE_WIDTH: i32 = 32;
pub const TILE_HEIGHT: i32 = 16;

#[derive(Component)]
struct Tile;

#[derive(Clone, Debug)]
pub enum FullTileType {
    Boundary { x: bool, z: bool },
    Stacked,
}

impl FullTileType {
    pub fn none() -> Self {
        Self::Boundary { x: false, z: false }
    }
    pub fn boundary(x: bool, z: bool) -> Self {
        Self::Boundary { x, z }
    }
    pub fn stacked() -> Self {
        Self::Stacked
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
    Full(FullTileType),
    Layer(FullTileType),
    SlopeLower { facing: TileFacing, has_edge: bool },
    SlopeUpper { facing: TileFacing, has_edge: bool },
    Stairs(TileFacing),
    Bridge(Option<TileFacing>),
}

impl TileType {
    fn get_collision(&self) -> Collider {
        match self {
            TileType::SlopeLower { facing, .. } => match facing {
                TileFacing::PosX => get_tile_collider_hull(0.5, 0.5, 0.0, 0.0),
                TileFacing::NegX => get_tile_collider_hull(0.0, 0.0, 0.5, 0.5),
                TileFacing::PosZ => get_tile_collider_hull(0.5, 0.0, 0.5, 0.0),
                TileFacing::NegZ => get_tile_collider_hull(0.0, 0.5, 0.0, 0.5),
            },
            TileType::SlopeUpper { facing, .. } => match facing {
                TileFacing::PosX => get_tile_collider_hull(1.0, 1.0, 0.5, 0.5),
                TileFacing::NegX => get_tile_collider_hull(0.5, 0.5, 1.0, 1.0),
                TileFacing::PosZ => get_tile_collider_hull(1.0, 0.5, 1.0, 0.5),
                TileFacing::NegZ => get_tile_collider_hull(0.5, 1.0, 0.5, 1.0),
            },
            TileType::Stairs(facing) => match facing {
                TileFacing::PosX => get_tile_collider_hull(1.0, 1.0, 0.0, 0.0),
                TileFacing::NegX => get_tile_collider_hull(0.0, 0.0, 1.0, 1.0),
                TileFacing::PosZ => get_tile_collider_hull(1.0, 0.0, 1.0, 0.0),
                TileFacing::NegZ => get_tile_collider_hull(0.0, 1.0, 0.0, 1.0),
            },
            _ => Collider::aabb(Vec3::ONE),
        }
    }

    fn get_atlas_index(&self) -> usize {
        match self {
            TileType::Full(tile_type) => match tile_type {
                FullTileType::Boundary { x, z } => match (x, z) {
                    (false, false) => 0,
                    (true, false) => 1,
                    (false, true) => 2,
                    (true, true) => 3,
                },
                FullTileType::Stacked => 4,
            },
            TileType::SlopeLower {
                facing,
                has_edge: false,
            } => match facing {
                TileFacing::NegX => 8,
                TileFacing::NegZ => 9,
                TileFacing::PosX => 10,
                TileFacing::PosZ => 11,
            },
            TileType::SlopeUpper {
                facing,
                has_edge: false,
            } => match facing {
                TileFacing::NegX => 12,
                TileFacing::NegZ => 13,
                TileFacing::PosX => 14,
                TileFacing::PosZ => 15,
            },
            TileType::SlopeLower {
                facing,
                has_edge: true,
            } => match facing {
                TileFacing::NegX => 16,
                TileFacing::NegZ => 17,
                TileFacing::PosX => 18,
                TileFacing::PosZ => 19,
            },
            TileType::SlopeUpper {
                facing,
                has_edge: true,
            } => match facing {
                TileFacing::NegX => 20,
                TileFacing::NegZ => 21,
                TileFacing::PosX => 22,
                TileFacing::PosZ => 23,
            },
            TileType::Stairs(facing) => match facing {
                TileFacing::NegX => 24,
                TileFacing::NegZ => 25,
                TileFacing::PosX => 26,
                TileFacing::PosZ => 27,
            },
            TileType::Bridge(facing) => match facing {
                Some(facing) => match facing {
                    TileFacing::NegX => 33,
                    TileFacing::NegZ => 34,
                    TileFacing::PosX => 35,
                    TileFacing::PosZ => 36,
                },
                None => 32,
            },
            TileType::Layer(tile_type) => match tile_type {
                FullTileType::Boundary { x, z } => match (x, z) {
                    (false, false) => 40,
                    (true, false) => 41,
                    (false, true) => 42,
                    (true, true) => 43,
                },
                FullTileType::Stacked => 44,
            },
        }
    }
}

fn get_tile_collider_hull(pp: f32, pn: f32, np: f32, nn: f32) -> Collider {
    Collider::hull(vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, nn, 0.0),
        
        Vec3::new(0.0, np, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, pn, 0.0),
        
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(1.0, pp, 1.0),
    ])
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
        Transform::from_translation(*Into::<ScreenCoords>::into(tile_coords.into())),
        // Physics
        tile_type.get_collision(),
        PhysicsData::Static,
        // Rendering
        Sprite::from_atlas_image(
            sprite_sheet.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: tile_type.get_atlas_index(),
            },
        ),
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
