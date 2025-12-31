//! Spawn the main level.

mod map;
mod palette;
mod room;
pub mod grid;

use bevy::prelude::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use crate::game::character::CharacterAssets;
use crate::game::character::animation::CharacterAnimationData;
use crate::game::character::player::{player, PlayerAssets};
use grid::coords::TileCoords;
use grid::grid;
use grid::tile::assets::{TileAssets, TileMaterial};
use grid::tile::tile_types::TileType;
use grid::tile::{tile, TileEdges, TileFacing, TileShape};
use crate::game::object::{object, ObjectAssets, ObjectType};
use crate::{asset_tracking::LoadResource, audio::music, screens::Screen, Scale};
use crate::game::level::map::temp_create_level;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();

    app.add_plugins((map::plugin, palette::plugin, room::plugin));
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
            music: assets.load("audio/music/8 Bit Open World.ogg"),
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
    _character_assets: Res<CharacterAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    animation_assets: Res<Assets<CharacterAnimationData>>,
) {
    let level = commands
        .spawn((
            Name::new("Level"),
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
            children![
                player(
                    Vec3::new(7.0, 1.0, 8.0),
                    //Vec3::new(0.0, 1.0, 0.0),
                    4.5,
                    &player_assets,
                    &mut texture_atlas_layouts,
                    &animation_assets,
                    scale.0
                ),
                (
                    Name::new("Gameplay Music"),
                    music(level_assets.music.clone())
                ),
                object(
                    ObjectType::Rock,
                    &object_assets,
                    Vec3::new(7.0, 5.0, 6.0),
                    scale.0,
                    0.5,
                    0.5,
                ),
            ],
        ))
        .id();

    let grid = temp_create_level(
        commands.reborrow(),
        scale,
        tile_assets,
        texture_atlas_layouts,
    );

    commands.entity(level).add_child(grid);
}

#[derive(Debug)]
pub struct TileSettingsParseError(String);
impl Error for TileSettingsParseError {}
impl std::fmt::Display for TileSettingsParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TileType {
    type Err = TileSettingsParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        println!("Parsing tile: {}", s);

        let mut parts = s.splitn(2, ':');
        let shape = parts
            .next()
            .ok_or_else(|| TileSettingsParseError("No tile type".to_string()))?
            .parse()?;
        let material = parts
            .next()
            .ok_or_else(|| TileSettingsParseError("No tile material".to_string()))?
            .parse()?;

        Ok(TileType::get_tile(shape, material))
    }
}

impl FromStr for TileShape {
    type Err = TileSettingsParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.replace("_", "");

        if s.is_empty() {
            return Err(TileSettingsParseError("No tile".to_string()));
        }

        let mut s = s.split('=');
        let tile_type = s.next().expect("No tile type");
        let data = s.next();

        let data = data.map(|s| s.split('|'));

        match tile_type {
            "F" | "L" | "B" => {
                let _edges = if let Some(mut data) = data.clone()
                    && let Some(first) = data.next()
                {
                    let mut edges = TileEdges::default();

                    if first.contains('X') {
                        edges += TileEdges::pos_x()
                    }
                    if first.contains('x') {
                        edges += TileEdges::neg_x()
                    }
                    if first.contains('Z') {
                        edges += TileEdges::pos_z()
                    }
                    if first.contains('z') {
                        edges += TileEdges::neg_z()
                    }

                    edges
                } else {
                    TileEdges::default()
                };

                match tile_type {
                    "F" => {
                        let is_top = if let Some(mut data) = data {
                            data.next();
                            if let Some(second) = data.next() {
                                !second.contains('S')
                            } else {
                                true
                            }
                        } else {
                            true
                        };

                        Ok(TileShape::Full { is_top })
                    }
                    "L" => {
                        let is_top = if let Some(mut data) = data {
                            data.next();
                            if let Some(second) = data.next() {
                                !second.contains('S')
                            } else {
                                true
                            }
                        } else {
                            true
                        };

                        Ok(TileShape::Layer { is_top })
                    }
                    "B" => {
                        let facing = if let Some(mut data) = data {
                            data.next();
                            if let Some(second) = data.next() {
                                match second {
                                    "X" => Some(TileFacing::PosX),
                                    "x" => Some(TileFacing::NegX),
                                    "Z" => Some(TileFacing::PosZ),
                                    "z" => Some(TileFacing::NegZ),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        Ok(TileShape::Bridge(facing))
                    }
                    _ => unreachable!(),
                }
            }
            "PL" => {
                let mut data = data.expect("Slopes require data");
                let facing = data.next().expect("Slopes require facing");
                let facing = match facing {
                    "X" => TileFacing::PosX,
                    "x" => TileFacing::NegX,
                    "Z" => TileFacing::PosZ,
                    "z" => TileFacing::NegZ,
                    _ => return Err(TileSettingsParseError("Invalid facing".to_string())),
                };

                let _has_edge = if let Some(second) = data.next()
                    && second.contains('E')
                {
                    true
                } else {
                    false
                };

                Ok(TileShape::SlopeLower(facing))
            }
            "PU" => {
                let mut data = data.expect("Slopes require data");
                let facing = data.next().expect("No facing");
                let facing = match facing {
                    "X" => TileFacing::PosX,
                    "x" => TileFacing::NegX,
                    "Z" => TileFacing::PosZ,
                    "z" => TileFacing::NegZ,
                    _ => return Err(TileSettingsParseError("Invalid facing".to_string())),
                };

                let _has_edge = if let Some(second) = data.next()
                    && second.contains('E')
                {
                    true
                } else {
                    false
                };

                Ok(TileShape::SlopeUpper(facing))
            }
            "S" => {
                let mut data = data.expect("Stairs require data");
                let facing = data.next().expect("No facing");
                let facing = match facing {
                    "X" => TileFacing::PosX,
                    "x" => TileFacing::NegX,
                    "Z" => TileFacing::PosZ,
                    "z" => TileFacing::NegZ,
                    _ => return Err(TileSettingsParseError("Invalid facing".to_string())),
                };

                Ok(TileShape::Stairs(facing))
            }

            _ => Err(TileSettingsParseError("Invalid tile type".to_string())),
        }
    }
}

impl FromStr for TileMaterial {
    type Err = TileSettingsParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.replace("_", "");

        match s.as_str() {
            "G" => Ok(TileMaterial::Grass),
            "P" => Ok(TileMaterial::DarkPlanks),
            "FP" => Ok(TileMaterial::DarkFramedPlanks),
            _ => Err(TileSettingsParseError("Invalid tile material".to_string())),
        }
    }
}
