//! Spawn the main level.

use bevy::prelude::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use crate::game::grid::coords::TileCoords;
use crate::game::grid::{FullTileType, TileAssets, TileFacing, TileMaterial, TileType, grid, tile};
use crate::game::object::{ObjectAssets, ObjectType, object};
use crate::{
    Scale,
    asset_tracking::LoadResource,
    audio::music,
    game::player::{PlayerAssets, player},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    scale: Res<Scale>,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    tile_assets: Res<TileAssets>,
    object_assets: Res<ObjectAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let level = commands
        .spawn((
            Name::new("Level"),
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
            children![
                player(
                    Vec3::new(3.0, 1.0, 3.0),
                    3.5,
                    &player_assets,
                    &mut texture_atlas_layouts,
                    scale.0
                ),
                (
                    Name::new("Gameplay Music"),
                    music(level_assets.music.clone())
                ),
                object(
                    ObjectType::Rock,
                    &object_assets,
                    Vec3::new(2.0, 1.0, 7.0),
                    scale.0,
                    Vec3::new(0.75, 0.5, 0.75),
                ),
            ],
        ))
        .id();

    let grid = create_level(
        commands.reborrow(),
        scale,
        tile_assets,
        texture_atlas_layouts,
    );

    commands.entity(level).add_child(grid);
}

fn create_level(
    mut commands: Commands,
    scale: Res<Scale>,
    tile_assets: Res<TileAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) -> Entity {
    let tile_map = Arc::new(RwLock::new(BTreeMap::<TileCoords, Entity>::new()));

    let level_layout_1 = [
        "_LB:G,_LZ:G,_LZ:G,_LZ:G,_LB:G,_LZ:G,_____,_____,_LB:G,_LZ:G,_LZ:G,_LZ:G,_LZ:G,_LB:G,_____,",
        "_LX:G,__L:G,__L:G,__L:G,__L:G,__L:G,_____,_____,_LX:G,__L:G,__L:G,__L:G,__L:G,_LX:G,_____,",
        "_LX:G,__L:G,__L:G,_LB:G,__L:G,__L:G,_____,_____,_LX:G,__L:G,__L:G,__L:G,_LX:G,__L:G,_____,",
        "_LX:G,__L:G,__L:G,_LX:G,__L:G,__L:G,_LZ:G,_LZ:G,_LZ:G,_LZ:G,_LZ:G,_LZ:G,__L:G,__L:G,_____,",
        "_LX:G,__L:G,__L:G,__L:G,__L:G,__L:G,_____,_____,_____,_____,_LX:P,_____,_____,_____,_____,",
        "_LX:G,__L:G,__L:G,__L:G,__L:G,__L:G,_____,_____,_____,_____,_LX:P,_LZ:P,_LZ:P,_____,_____,",
        "_LX:G,__L:G,__L:G,__L:G,__L:G,_LX:G,_____,_____,_____,_____,_____,_____,_LX:P,_____,_____,",
        "_LX:G,__L:G,__L:G,__L:G,_LZ:G,__L:G,_____,_____,_____,_____,_____,_____,_LX:P,_LZ:P,_LZ:P,",
        "_LX:G,__L:G,__L:G,__L:G,__L:G,__L:G,_____,_____,_____,_____,_____,_____,_LX:P,_LX:P,__L:P,",
        "_LS:G,_LS:G,_LX:G,__L:G,__L:G,__L:G,_____,_____,_____,_____,_____,_____,_LS:P,_LX:P,__L:P,",
        "_LS:G,_LS:G,_LX:G,__L:G,__L:G,__L:G,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
    ];
    let level_layout_2 = [
        "_FB:G,_FZ:G,_FZ:G,_FS:P,_____,_____,_____,_____,_FB:P,_FZ:P,_FZ:P,_FZ:P,_FZ:P,_____,_____,",
        "_FX:G,__F:G,__F:G,S-x:G,_____,_____,_____,_____,_FX:P,__F:P,__F:P,__F:P,S-z:P,_____,_____,",
        "_____,O-z:G,P-z:G,_____,_____,_____,_____,_____,_FX:P,__F:P,__F:P,__F:P,_____,_____,_____,",
        "_____,o-z:G,p-z:G,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_FB:G,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,S+z:P,_____,_____,",
        "S+z:G,S+z:G,_____,_____,_____,S+x:P,B-x:P,__B:P,__B:P,__B:P,__B:P,B+x:P,__F:P,_____,_____,",
        "_FX:G,__F:G,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
    ];
    let level_layout_3 = [
        "_____,_____,_____,S+x:P,B-x:P,__B:P,__B:P,B+x:P,S-x:P,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
    ];

    let level_layout = [level_layout_1, level_layout_2, level_layout_3];

    let mut tile_coords = Vec::new();

    for (y, layer) in level_layout.into_iter().enumerate() {
        for (z, row) in layer.into_iter().enumerate() {
            let chars = row.split(',');

            for (x, col) in chars.into_iter().enumerate() {
                let result = col.parse::<TileSettings>();
                if let Ok(tile_settings) = result {
                    tile_coords.push((
                        tile_settings.tile_material,
                        tile_settings.tile_type,
                        TileCoords(IVec3::new(x as i32, y as i32, z as i32)),
                    ));
                } else if !col.replace("_", "").is_empty() && !col.replace(" ", "").is_empty() {
                    println!("Invalid tile settings: {}", col);
                }
            }
        }
    }

    let grid = grid(tile_map.clone(), scale.0);
    let grid = commands.spawn(grid).id();

    for (material, tile_type, coords) in tile_coords {
        let tile = commands
            .spawn(tile(
                tile_type,
                material,
                coords.clone(),
                &tile_assets,
                &mut texture_atlas_layouts,
            ))
            .id();

        commands.entity(grid).add_child(tile);

        tile_map.write().unwrap().insert(coords, tile);
    }

    grid
}

#[derive(Debug)]
pub struct TileSettingsParseError(String);
impl Error for TileSettingsParseError {}
impl std::fmt::Display for TileSettingsParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct TileSettings {
    tile_type: TileType,
    tile_material: TileMaterial,
}

impl FromStr for TileSettings {
    type Err = TileSettingsParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut parts = s.splitn(2, ':');
        let tile_type = parts
            .next()
            .ok_or_else(|| TileSettingsParseError("No tile type".to_string()))?
            .parse()?;
        let tile_material = parts
            .next()
            .ok_or_else(|| TileSettingsParseError("No tile material".to_string()))?
            .parse()?;

        Ok(Self {
            tile_type,
            tile_material,
        })
    }
}

impl FromStr for TileType {
    type Err = TileSettingsParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.replace("_", "");

        match s.as_str() {
            "F" => Ok(TileType::Full(FullTileType::none())),
            "FX" => Ok(TileType::Full(FullTileType::boundary(true, false))),
            "FZ" => Ok(TileType::Full(FullTileType::boundary(false, true))),
            "FB" => Ok(TileType::Full(FullTileType::boundary(true, true))),
            "FS" => Ok(TileType::Full(FullTileType::stacked())),

            "p+x" => Ok(TileType::SlopeLower {
                facing: TileFacing::PosX,
                has_edge: false,
            }),
            "p-x" => Ok(TileType::SlopeLower {
                facing: TileFacing::NegX,
                has_edge: false,
            }),
            "p+z" => Ok(TileType::SlopeLower {
                facing: TileFacing::PosZ,
                has_edge: false,
            }),
            "p-z" => Ok(TileType::SlopeLower {
                facing: TileFacing::NegZ,
                has_edge: false,
            }),
            "o+x" => Ok(TileType::SlopeLower {
                facing: TileFacing::PosX,
                has_edge: true,
            }),
            "o-x" => Ok(TileType::SlopeLower {
                facing: TileFacing::NegX,
                has_edge: true,
            }),
            "o+z" => Ok(TileType::SlopeLower {
                facing: TileFacing::PosZ,
                has_edge: true,
            }),
            "o-z" => Ok(TileType::SlopeLower {
                facing: TileFacing::NegZ,
                has_edge: true,
            }),

            "P+x" => Ok(TileType::SlopeUpper {
                facing: TileFacing::PosX,
                has_edge: false,
            }),
            "P-x" => Ok(TileType::SlopeUpper {
                facing: TileFacing::NegX,
                has_edge: false,
            }),
            "P+z" => Ok(TileType::SlopeUpper {
                facing: TileFacing::PosZ,
                has_edge: false,
            }),
            "P-z" => Ok(TileType::SlopeUpper {
                facing: TileFacing::NegZ,
                has_edge: false,
            }),
            "O+x" => Ok(TileType::SlopeUpper {
                facing: TileFacing::PosX,
                has_edge: true,
            }),
            "O-x" => Ok(TileType::SlopeUpper {
                facing: TileFacing::NegX,
                has_edge: true,
            }),
            "O+z" => Ok(TileType::SlopeUpper {
                facing: TileFacing::PosZ,
                has_edge: true,
            }),
            "O-z" => Ok(TileType::SlopeUpper {
                facing: TileFacing::NegZ,
                has_edge: true,
            }),

            "S+x" => Ok(TileType::Stairs(TileFacing::PosX)),
            "S-x" => Ok(TileType::Stairs(TileFacing::NegX)),
            "S+z" => Ok(TileType::Stairs(TileFacing::PosZ)),
            "S-z" => Ok(TileType::Stairs(TileFacing::NegZ)),

            "B" => Ok(TileType::Bridge(None)),
            "B+x" => Ok(TileType::Bridge(Some(TileFacing::PosX))),
            "B-x" => Ok(TileType::Bridge(Some(TileFacing::NegX))),
            "B+z" => Ok(TileType::Bridge(Some(TileFacing::PosZ))),
            "B-z" => Ok(TileType::Bridge(Some(TileFacing::NegZ))),

            "L" => Ok(TileType::Layer(FullTileType::none())),
            "LX" => Ok(TileType::Layer(FullTileType::boundary(true, false))),
            "LZ" => Ok(TileType::Layer(FullTileType::boundary(false, true))),
            "LB" => Ok(TileType::Layer(FullTileType::boundary(true, true))),
            "LS" => Ok(TileType::Layer(FullTileType::stacked())),

            _ => Err(TileSettingsParseError("Invalid tile type".to_string())),
        }
    }
}

impl FromStr for TileMaterial {
    type Err = TileSettingsParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "G" => Ok(TileMaterial::Grass),
            "S" => Ok(TileMaterial::Stone),
            "P" => Ok(TileMaterial::Planks),
            _ => Err(TileSettingsParseError("Invalid tile material".to_string())),
        }
    }
}
