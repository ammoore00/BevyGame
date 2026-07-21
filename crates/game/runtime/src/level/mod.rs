//! Spawn the main level.

pub mod grid;
pub mod map;

use crate::character::npc::npc_bundle;
use crate::character::player::player;
use crate::level::grid::nav::NavContext;
use crate::level::map::{map_scene, NavBakeError};
use assets::resource::level::{Palette, Palettes};
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;
use common::{marker, GameState};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((grid::plugin, map::plugin));

    app.init_state::<LevelSpawnState>();
    app.add_systems(OnEnter(LevelSpawnState::ConstructLevel), construct_level);
    app.add_systems(OnEnter(LevelSpawnState::BakeTiles), bake_tiles);
    app.add_systems(OnEnter(LevelSpawnState::BakeNav), bake_nav);
    app.add_systems(OnEnter(LevelSpawnState::AddObjects), add_objects);
    app.add_systems(OnEnter(LevelSpawnState::Cleanup), finish_level_spawn);

    app.add_observer(on_spawn_level);
    app.add_observer(on_reset_level);
    app.add_observer(on_level_error);

    app.configure_sets(Update, LevelLoadedSystems.run_if(in_state(LevelSpawnState::Finished)));
}

/// Systems that should run only after the level has been loaded
#[derive(SystemSet, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelLoadedSystems;

marker!(Level);

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelSpawnState {
    #[default]
    Uninitialized,
    ConstructLevel,
    BakeTiles,
    BakeNav,
    AddObjects,
    Cleanup,
    Finished,
    Error,
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
            LevelSpawnState::Error=> {
                error!("Attempting to transition from Error state using next().");
                LevelSpawnState::Error
            },
        }
    }
}

#[derive(Event)]
pub struct SpawnLevelEvent;
#[derive(Event)]
pub struct ResetLevelEvent;

fn on_spawn_level(
    _: On<SpawnLevelEvent>,
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

fn on_reset_level(
    _: On<ResetLevelEvent>,
    mut next_state: ResMut<NextState<LevelSpawnState>>,
) {
    info!("Cleaning up level");
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
        DespawnOnExit<GameState>(GameState::Gameplay)
        Children [
            // TODO: Fix this
            //music(loc::<AudioResource>("music/8_bit_open_world").unwrap())
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
            error!("Error getting level entity while baking tiles: {:?}", err);
            next_state.set(LevelSpawnState::Error);
            commands.trigger(LevelErrorEvent(err.into()));
            return;
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
    mut commands: Commands,
) {
    info!("Level construction - baking nav level");

    if let Err(err) = map::bake_nav(nav_context, commands.reborrow()) {
        error!("Error baking nav level: {:?}", err);
        commands.trigger(LevelErrorEvent(err.into()));
        return;
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
            error!("Error getting level entity while adding objects: {:?}", err);
            next_state.set(LevelSpawnState::Error);
            commands.trigger(LevelErrorEvent(err.into()));
            return;
        },
    };

    let player = player(Vec3::new(3.0, 1.0, 3.0));

    let test_npc = npc_bundle(
        "test".parse().unwrap(),
        Vec3::new(5.0, 1.0, 3.0)
    );

    let children = &[
        commands.spawn_scene(player).id(),
        //commands.spawn_scene(test_npc).id(),
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

fn on_level_error(event: On<LevelErrorEvent>) {
    error!("Error encountered while building level: {:?}", event.0);
    todo!()
}

#[derive(Event, Debug)]
pub struct LevelErrorEvent(LevelError);

#[derive(thiserror::Error, Debug)]
enum LevelError {
    #[error("Failed to get level entity: {}", .0)]
    LevelEntity(#[from] QuerySingleError),
    #[error("Failed to bake navmesh: {}", .0)]
    NavBaking(#[from] NavBakeError),
}