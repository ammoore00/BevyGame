//! Spawn the main level.

pub mod grid;
pub mod map;

use crate::audio::AudioResource;
use crate::data::loc;
use crate::game::character::npc::npc_bundle;
use crate::game::character::player::player;
use crate::game::level::grid::nav::NavContext;
use crate::game::level::map::palette::{Palette, Palettes};
use crate::game::level::map::map_scene;
use crate::{audio::music, marker, screens::Screen};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((grid::plugin, map::plugin));

    app.init_state::<LevelSpawnState>();
    app.add_systems(OnEnter(LevelSpawnState::ConstructLevel), construct_level);
    app.add_systems(OnEnter(LevelSpawnState::BakeTiles), bake_tiles);
    app.add_systems(OnEnter(LevelSpawnState::BakeNav), bake_nav);
    app.add_systems(OnEnter(LevelSpawnState::AddObjects), add_objects);
    app.add_systems(OnEnter(LevelSpawnState::Cleanup), finish_level_spawn);
}

marker!(Level);

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LevelSpawnState {
    #[default]
    Uninitialized,
    ConstructLevel,
    BakeTiles,
    BakeNav,
    AddObjects,
    Cleanup,
    Finished,
}
impl LevelSpawnState {
    fn next(self) -> Self {
        match self {
            LevelSpawnState::Uninitialized => LevelSpawnState::ConstructLevel,
            LevelSpawnState::ConstructLevel => LevelSpawnState::BakeTiles,
            LevelSpawnState::BakeTiles => LevelSpawnState::BakeNav,
            LevelSpawnState::BakeNav => LevelSpawnState::AddObjects,
            LevelSpawnState::AddObjects => LevelSpawnState::Cleanup,
            LevelSpawnState::Cleanup => LevelSpawnState::Finished,
            LevelSpawnState::Finished => {
                warn!("Attempting to transition from Finished state using next().");
                LevelSpawnState::Finished
            },
        }
    }
}

pub fn spawn_level(
    prev_state: Res<State<LevelSpawnState>>,
    mut next_state: ResMut<NextState<LevelSpawnState>>,
) {
    if **prev_state != LevelSpawnState::Uninitialized {
        error!("Attempting to spawn level while level construction is in progress or finished!");
        return;
    }

    info!("Beginning level construction!");
    next_state.set(LevelSpawnState::Uninitialized.next());
}

pub fn reset_level_state(mut next_state: ResMut<NextState<LevelSpawnState>>) {
    next_state.set(LevelSpawnState::Uninitialized);
}

fn construct_level(
    mut next_state: ResMut<NextState<LevelSpawnState>>,
    mut commands: Commands,
) {
    info!("Level construction - init");

    let level = bsn![
        #Level
        Level
        Transform
        Visibility
        DespawnOnExit<Screen>(Screen::Gameplay)
        Children [
            music(loc::<AudioResource>("music/8_bit_open_world").unwrap())
        ]
    ];
    commands.spawn_scene(level);
    next_state.set(LevelSpawnState::ConstructLevel.next());
}
fn bake_tiles(
    level: Query<Entity, With<Level>>,
    level_palettes: Res<Palettes>,
    palette_assets: Res<Assets<Palette>>,
    mut next_state: ResMut<NextState<LevelSpawnState>>,
    mut commands: Commands,
) {
    info!("Level construction - baking tiles");

    let level = match level.single() {
        Ok(level) => level,
        Err(err) => {
            // TODO: figure out how to recover from this error
            panic!("Error getting level entity: {:?}", err);
        },
    };

    let palettes = level_palettes.into_inner();
    let palette = palette_assets.get(palettes.standard.id()).unwrap();

    let map = &palette.main_map_pool().0[0];

    let map = commands.spawn_scene(map_scene(map, palette)).id();
    commands.entity(level).add_child(map);

    next_state.set(LevelSpawnState::BakeTiles.next());
}
fn bake_nav(
    nav_context: NavContext,
    mut next_state: ResMut<NextState<LevelSpawnState>>,
    commands: Commands,
) {
    info!("Level construction - baking nav map");

    if let Err(err) = map::bake_nav(nav_context, commands) {
        // TODO: figure out how to recover from this error
        panic!("Error baking nav map: {:?}", err);
    };

    next_state.set(LevelSpawnState::BakeNav.next());
}
fn add_objects(
    level: Query<Entity, With<Level>>,
    mut next_state: ResMut<NextState<LevelSpawnState>>,
    mut commands: Commands,
) {
    info!("Level construction - adding objects");

    let level = match level.single() {
        Ok(level) => level,
        Err(err) => {
            // TODO: figure out how to recover from this error
            panic!("Error getting level entity: {:?}", err);
        },
    };

    let player = player(Vec3::new(3.0, 1.0, 3.0));

    let test_npc = npc_bundle(
        "test".parse().unwrap(),
        Vec3::new(5.0, 1.0, 3.0)
    );

    let children = &[
        commands.spawn_scene(player).id(),
        commands.spawn_scene(test_npc).id(),
    ];
    commands.entity(level).add_children(children);

    next_state.set(LevelSpawnState::AddObjects.next());
}
fn finish_level_spawn(
    mut next_state: ResMut<NextState<LevelSpawnState>>,
) {
    info!("Level construction - cleanup");
    next_state.set(LevelSpawnState::Cleanup.next());
    info!("Finished constructing level");
}