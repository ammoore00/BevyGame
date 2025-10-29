//! Spawn the main level.

use bevy::prelude::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use crate::game::grid::coords::TileCoords;
use crate::game::grid::{TileAssets, TileFacing, TileMaterial, TileType, grid, tile};
use crate::game::object::{ColliderType, ObjectAssets, ObjectType, object};
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
                    Vec3::new(2.0, 0.0, 3.0),
                    3.5,
                    &player_assets,
                    &mut texture_atlas_layouts,
                    scale.0),
                (
                    Name::new("Gameplay Music"),
                    music(level_assets.music.clone())
                ),
                object(
                    ObjectType::Rock,
                    &object_assets,
                    Vec3::new(1.75, 0.0, 7.75),
                    scale.0,
                    ColliderType::Cylinder {
                        radius: 0.375,
                        height: 0.5
                    }
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
        "__L:G,__L:G,__L:G,__L:G,__L:G,_____,_____,__L:G,__L:G,__L:G,__L:G,__L:G,__L:G,_____,",
        "__L:G,__L:G,__L:G,__L:G,__L:G,_____,_____,__L:G,__L:G,__L:G,__L:G,__L:G,__L:G,_____,",
        "__L:G,__L:G,__L:G,__L:G,__L:G,_____,_____,__L:G,__L:G,__L:G,__L:G,__L:G,__L:G,_____,",
        "__L:G,__L:G,__L:G,__L:G,__L:G,__L:G,__L:G,__L:G,__L:G,__L:G,__L:G,__L:G,__L:G,_____,",
        "_LS:G,__L:G,__L:G,__L:G,__L:G,_____,_____,_____,_____,__L:P,_____,_____,_____,_____,",
        "_____,_____,__L:G,_____,_____,_____,_____,_____,_____,__L:P,__L:P,__L:P,_____,_____,",
        "__L:G,__L:G,__L:G,__L:G,__L:G,_____,_____,_____,_____,_____,_____,__L:P,_____,_____,",
        "__L:G,__L:G,__L:G,__L:G,__L:G,_____,_____,_____,_____,_____,_____,__L:P,__L:P,__L:P,",
        "__L:G,__L:G,__L:G,__L:G,__L:G,_____,_____,_____,_____,_____,_____,__L:P,__L:P,__L:P,",
        "_LS:G,__L:G,__L:G,__L:G,__L:G,_____,_____,_____,_____,_____,_____,_LS:P,__L:P,__L:P,",
        "_LS:G,__L:G,__L:G,__L:G,__L:G,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
    ];
    let level_layout_2 = [
        "__F:G,__F:G,_FS:P,_____,_____,_____,_____,__F:P,__F:P,__F:P,__F:P,__F:P,_____,_____,",
        "__F:G,__F:G,S-x:G,_____,_____,_____,_____,__F:P,__F:P,__F:P,__F:P,S-z:P,_____,_____,",
        "__F:G,__F:G,_____,_____,_____,_____,_____,__F:P,__F:P,__F:P,__F:P,_____,_____,_____,",
        "__F:G,S-z:G,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "__F:G,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,S+z:P,_____,_____,",
        "S+z:G,_____,_____,_____,S+x:P,B-x:P,__B:P,__B:P,__B:P,__B:P,B+x:P,__F:P,_____,_____,",
        "__F:G,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
    ];
    let level_layout_3 = [
        "_____,_____,S+x:P,B-x:P,__B:P,__B:P,B+x:P,S-x:P,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
        "_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,_____,",
    ];

    let level_layout = [
        level_layout_1,
        level_layout_2,
        level_layout_3,
    ];

    let mut tile_coords = Vec::new();

    for (y, layer) in level_layout.into_iter().enumerate() {
        for (z, row) in layer.into_iter().enumerate() {
            let chars = row.split(',');

            for (x, col) in chars.into_iter().enumerate() {
                if let Ok(tile_settings) = col.parse::<TileSettings>() {
                    tile_coords.push((
                        tile_settings.tile_material,
                        tile_settings.tile_type,
                        TileCoords(IVec3::new(x as i32, y as i32, z as i32)),
                    ));
                }
            }
        }
    }

    let grid = grid(
        tile_map.clone(),
        tile_assets.clone(),
        scale.0,
        &mut texture_atlas_layouts,
    );
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
            "F" => Ok(TileType::Full),
            "FS" => Ok(TileType::FullStacked),
            "L" => Ok(TileType::Layer),
            "LS" => Ok(TileType::LayerStacked),
            "B" => Ok(TileType::Bridge(None)),

            "B+x" => Ok(TileType::Bridge(Some(TileFacing::PosX))),
            "B-x" => Ok(TileType::Bridge(Some(TileFacing::NegX))),
            "B+z" => Ok(TileType::Bridge(Some(TileFacing::PosZ))),
            "B-z" => Ok(TileType::Bridge(Some(TileFacing::NegZ))),

            "S+x" => Ok(TileType::Stairs(TileFacing::PosX)),
            "S-x" => Ok(TileType::Stairs(TileFacing::NegX)),
            "S+z" => Ok(TileType::Stairs(TileFacing::PosZ)),
            "S-z" => Ok(TileType::Stairs(TileFacing::NegZ)),

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
