use crate::debug_options::options::TileCollisionRes;
use bevy::prelude::*;
use common::dev_tools::DebugState;
use common::marker;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_tile_collision_render.run_if(resource_changed::<TileCollisionRes>),
        )
    );

    app.add_observer(spawn_tile_collision_render);
    app.add_observer(cleanup_tile_collision_render);
}

marker!(TileCollisionRender);

#[derive(Event)]
struct SpawnTileCollision;
#[derive(Event)]
struct CleanupTileCollision;

fn update_tile_collision_render(
    render_tile_collision: Res<TileCollisionRes>,
    mut commands: Commands,
) {
    if render_tile_collision.get() {
        commands.trigger(SpawnTileCollision);
    } else {
        commands.trigger(CleanupTileCollision);
    }
}

fn spawn_tile_collision_render(
    _: On<SpawnTileCollision>,
    mut commands: Commands,
) {
    
}

fn cleanup_tile_collision_render(
    _: On<CleanupTileCollision>,
    render_query: Query<Entity, With<TileCollisionRender>>,
    mut commands: Commands,
) {
    for entity in render_query.iter() {
        commands.entity(entity).despawn();
    }
}