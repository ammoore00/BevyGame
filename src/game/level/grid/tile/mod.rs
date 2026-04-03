use crate::game::level::grid::coords::{ScreenCoords, TileCoords, TilePosition, WorldCoords};
use crate::game::level::grid::tile::assets::TileAssets;
use crate::game::level::grid::tile::tile_types::TileType;
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::image::TextureAtlas;
use bevy::math::Vec3;
use bevy::prelude::*;
use std::fmt::Debug;
use std::ops::{Add, AddAssign};

pub mod assets;
mod collision;
pub mod tile_types;

pub fn plugin(app: &mut App) {
    app.add_plugins((assets::plugin, codec::plugin));
    app.add_systems(
        Update,
        update_tile_collision
    );
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
pub struct Tile;
/// Newtype wrapper for better API clarity
pub struct TileEntity(pub Entity);

pub fn set_tile_location(
    tile: TileEntity, tile_coords: impl Into<TileCoords> + Clone, commands: &mut Commands
) {
    commands.entity(tile.0).insert(TilePosition(tile_coords.clone().into()));
}

fn update_tile_collision(
    tile_query: Query<(&TilePosition, &mut Collider), With<Tile>>,
) {
    for (tile_pos, mut collider) in tile_query {
        let world_coords = Into::<WorldCoords>::into(tile_pos.0.clone());
        collider.set_position(world_coords);
    }
}

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

pub mod codec {
    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};
    use crate::data::{ResourceFileType, ResourceLocation, ResourceType};
    use crate::data::loader::{LoaderJobManager, RonAssetLoader};
    use crate::data::registry::ResourceRegistry;
    use crate::data::sprite::SpriteResource;
    use crate::datagen_api::tile::collision;
    use crate::game::level::grid::coords::WorldCoords;
    use crate::game::physics::components::Collider;
    
    pub type TileRegistry = ResourceRegistry<TileResource, TileAsset>;

    pub fn plugin(app: &mut App) {
        app.init_asset_loader::<RonAssetLoader<TileCodec, TileAsset>>();
        app.init_asset::<TileAsset>();
        app.add_resource_registry::<TileResource, TileAsset>();
    }

    #[derive(derive_new::new, Serialize, Deserialize)]
    pub struct TileCodec {
        pub format: u8,
        pub sprite_sheet: ResourceLocation<SpriteResource>,
        pub sprite_index: u8,
    }

    #[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Default, Reflect)]
    pub struct TileResource;
    impl ResourceType for TileResource {
        fn root_dir() -> &'static str {
            "tiles"
        }

        fn file_type() -> ResourceFileType {
            ResourceFileType::Data
        }
    }

    #[derive(Debug, Clone, Asset, TypePath)]
    pub struct TileAsset {
        sprite_sheet: ResourceLocation<SpriteResource>,
        sprite_index: u8,
    }
    impl From<TileCodec> for TileAsset {
        fn from(codec: TileCodec) -> Self {
            Self {
                sprite_sheet: codec.sprite_sheet,
                sprite_index: codec.sprite_index,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
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
        Other(Vec<Vec3>),
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
                (TileShape::Other(a), TileShape::Other(b)) => {
                    let sorted = |v: &[Vec3]| -> Vec<[f32; 3]> {
                        let mut s: Vec<[f32; 3]> = v.iter()
                            .map(|v| [v.x, v.y, v.z])
                            .collect();
                        
                        s.sort_by(|a, b| {
                            a[0].total_cmp(&b[0])
                                .then(a[1].total_cmp(&b[1]))
                                .then(a[2].total_cmp(&b[2]))
                        });
                        
                        s
                    };
                    
                    let mut a = a.clone();
                    a.dedup();

                    let mut b = b.clone();
                    b.dedup();

                    a.len() == b.len() && sorted(&a) == sorted(&b)
                },
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
        pub(super) fn get_collision(&self, position: impl Into<WorldCoords>) -> Collider {
            let position = position.into();

            match self {
                TileShape::SlopeLower(facing) => collision::slope(0.0, 0.5, *facing)(position),
                TileShape::SlopeUpper(facing) => collision::slope(0.5, 1.0, *facing)(position),
                TileShape::Stairs(facing) => collision::slope_45(*facing)(position),
                TileShape::Bridge(_) => {
                    collision::cuboid(Vec3::new(0.5, 0.25, 0.5))((*position + Vec3::Y * 0.25).into())
                }
                TileShape::Other(points) => collision::convex_hull(points)(position),
                _ => collision::full()(position),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
}