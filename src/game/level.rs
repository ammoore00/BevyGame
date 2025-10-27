//! Spawn the main level.

use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, audio::music, game::player::{PlayerAssets, player}, screens::Screen, Scale};
use crate::game::grid::{grid, TileAssets};
use crate::game::object::{object, ObjectAssets, ObjectType};

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
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    tile_assets: Res<TileAssets>,
    object_assets: Res<ObjectAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    scale: Res<Scale>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![
            player(3.5, &player_assets, &mut texture_atlas_layouts, scale.0),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            ),
            grid(tile_assets.clone(), scale.0),
            object(ObjectType::Rock, &object_assets, -1.25, 1.0, -1.25, scale.0),
        ],
    ));
}
