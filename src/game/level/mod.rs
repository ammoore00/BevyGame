//! Spawn the main level.

pub mod grid;
pub mod map;

use crate::game::character::player::{player_bundle, PlayerAssets};
use crate::game::character::CharacterBuilderContext;
use crate::game::level::map::palette::{Palette, Palettes};
use crate::game::level::map::room::RoomBuilderContext;
use crate::game::object::{object_bundle, ObjectAssets, ObjectType};
use crate::{asset_tracking::LoadResource, audio::music, screens::Screen, Scale};
use bevy::prelude::*;
use crate::game::character::npc::npc_bundle;
use crate::game::level::grid::Grid;
use crate::game::level::grid::nav::{TileNavMap, TileNavQuery};
use crate::game::level::map::{build_map_grid, map_bundle, Map};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((grid::plugin, map::plugin));

    app.load_resource::<LevelAssets>();

    app.init_state::<LevelSpawnState>();
    app.add_systems(OnEnter(LevelSpawnState::ConstructLevel), construct_level);
    app.add_systems(OnEnter(LevelSpawnState::BakeTiles), bake_tiles);
    app.add_systems(OnEnter(LevelSpawnState::BakeNav), bake_nav);
    app.add_systems(OnEnter(LevelSpawnState::AddObjects), add_objects);
    app.add_systems(OnEnter(LevelSpawnState::Cleanup), finish_level_spawn);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Level;

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
    level_assets: Res<LevelAssets>,
    mut next_state: ResMut<NextState<LevelSpawnState>>,
    mut commands: Commands,
) {
    info!("Level construction - init");

    commands
        .spawn((
            Name::new("Level"),
            Level,
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
            children![
                (
                    Name::new("Gameplay Music"),
                    music(level_assets.music.clone())
                ),
                //rock,
            ],
        ));

    next_state.set(LevelSpawnState::ConstructLevel.next());
}
fn bake_tiles(
    level: Query<Entity, With<Level>>,
    level_palettes: Res<Palettes>,
    palette_assets: Res<Assets<Palette>>,
    mut builder_context: RoomBuilderContext,
    mut next_state: ResMut<NextState<LevelSpawnState>>,
    mut commands: Commands,
) {
    info!("Level construction - baking tiles");

    let level = match level.single() {
        Ok(level) => level,
        Err(err) => {
            error!("Error getting level entity: {:?}", err);
            return
        },
    };

    let palettes = level_palettes.into_inner();
    let palette = palette_assets.get(palettes.standard.id()).unwrap();

    let map = &palette.main_map_pool().0[0];
    let rng = rand::rng();

    // TODO: Move this responsibility into map module
    let grid_entity = build_map_grid(
        map,
        rng,
        palette,
        &mut builder_context,
    );

    let map_entity = commands
        .spawn(map_bundle())
        .add_child(grid_entity)
        .id();
    commands.entity(level).add_child(map_entity);

    next_state.set(LevelSpawnState::BakeTiles.next());
}
fn bake_nav(
    nav_query: TileNavQuery,
    map: Query<(Entity, &Children), With<Map>>,
    grid: Query<&Grid>,
    mut next_state: ResMut<NextState<LevelSpawnState>>,
    mut commands: Commands,
) {
    info!("Level construction - baking nav map");

    let (map_entity, children) = match map.single() {
        Ok(level) => level,
        Err(err) => {
            error!("Error getting level entity: {:?}", err);
            return
        },
    };

    let mut tile_map = None;
    for child in children {
        if let Ok(grid) = grid.get(*child) {
            tile_map = Some(grid.tile_map().clone());
            break;
        }
    }
    let Some(tile_map) = tile_map else {
        error!("No grid child found for level map");
        return;
    };

    // TODO: Move this responsibility into map module
    let tile_nav_map = TileNavMap::from_map(tile_map, nav_query);
    let nav_entity = commands.spawn(tile_nav_map).id();
    commands.entity(map_entity).add_child(nav_entity);

    next_state.set(LevelSpawnState::BakeNav.next());
}
fn add_objects(
    level: Query<Entity, With<Level>>,
    scale: Res<Scale>,
    player_assets: Res<PlayerAssets>,
    object_assets: Res<ObjectAssets>,
    character_context: CharacterBuilderContext,
    mut next_state: ResMut<NextState<LevelSpawnState>>,
    mut commands: Commands,
) {
    info!("Level construction - adding objects");

    let level = match level.single() {
        Ok(level) => level,
        Err(err) => {
            error!("Error getting level entity: {:?}", err);
            return
        },
    };

    let player = player_bundle(
        Vec3::new(3.0, 1.0, 3.0),
        4.5,
        &player_assets,
        scale.0,
        &character_context,
    );

    let test_npc = npc_bundle(
        "test".parse().unwrap(),
        Vec3::new(5.0, 1.0, 3.0),
        scale.0,
        &character_context,
    );

    let _rock = object_bundle(
        ObjectType::Rock,
        &object_assets,
        Vec3::new(6.0, 5.0, 6.0),
        scale.0,
        0.5,
        0.5,
    );

    let children = &[
        commands.spawn(player).id(),
        commands.spawn(test_npc).id(),
        //commands.spawn(_rock).id(),
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