//! Spawn the main level.

pub mod grid;
pub mod map;

use bevy::prelude::*;
use crate::game::character::animation::ResolvedAnimationData;
use crate::game::character::player::{PlayerAssets, player};
use crate::game::object::{ObjectAssets, ObjectType, object};
use crate::{Scale, asset_tracking::LoadResource, audio::music, screens::Screen};
use crate::game::character::CharacterBuilderContext;
use crate::game::level::map::palette::{Palette, Palettes};
use crate::game::level::map::room::RoomBuilderContext;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();

    app.add_plugins((grid::plugin, map::plugin));
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
            music: assets.load("base/audio/music/8 Bit Open World.ogg"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    scale: Res<Scale>,

    level_assets: Res<LevelAssets>,
    level_palettes: Res<Palettes>,
    palette_assets: Res<Assets<Palette>>,

    mut builder_context: RoomBuilderContext,
    character_context: CharacterBuilderContext,

    player_assets: Res<PlayerAssets>,
    object_assets: Res<ObjectAssets>,
    animation_assets: Res<Assets<ResolvedAnimationData>>,
) {
    let player = player(
        Vec3::new(3.0, 1.0, 3.0),
        //Vec3::new(0.0, 1.0, 0.0),
        4.5,
        &player_assets,
        &animation_assets,
        scale.0,
        &character_context,
    );

    let _rock = object(
        ObjectType::Rock,
        &object_assets,
        Vec3::new(7.0, 5.0, 6.0),
        scale.0,
        0.5,
        0.5,
    );

    let level = builder_context.commands
        .spawn((
            Name::new("Level"),
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
            children![
                player,
                (
                    Name::new("Gameplay Music"),
                    music(level_assets.music.clone())
                ),
                //rock,
            ],
        ))
        .id();

    let palettes = level_palettes.into_inner();
    let palette = palette_assets.get(palettes.standard.id()).unwrap();
    
    let map = &palette.main_map_pool().0[0];
    let rng = rand::rng();
    let map_state = map.bake(
        rng,
        palette,
        &mut builder_context,
    );
    builder_context.commands.entity(level).add_child(map_state.grid());
}