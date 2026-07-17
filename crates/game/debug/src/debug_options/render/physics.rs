use crate::debug_options::options::{CharacterCollisionRes, TileCollisionRes};
use crate::debug_options::render::helpers::*;
use crate::debug_options::render::palette::*;
use bevy::prelude::*;
use common::dev_tools::DebugState;
use common::{marker, Scale, TilePosition, WorldPosition};
use physics::{Collider, ColliderType};
use runtime::character::Character;
use runtime::debug::Tile;

pub(super) fn plugin(app: &mut App) {
    // Tile collision uses retained state for rendering because tile collision does not move
    // Character collision does move, and so it uses immediate mode rendering

    app.add_systems(
        Update,
        (
            update_character_collision_render,
            update_tile_collision_render.run_if(resource_changed::<TileCollisionRes>),
        ),
    );

    app.add_observer(spawn_tile_collision_render);
    app.add_observer(cleanup_tile_collision_render);
}

//------ Character Collision ------//

marker!(CharacterCollisionRender);

fn update_character_collision_render(
    character_query: Query<(&Collider, &WorldPosition), With<Character>>,
    render_query: Query<Entity, With<CharacterCollisionRender>>,
    scale: Res<Scale>,
    should_render_tile_collision: Res<CharacterCollisionRes>,
    mut commands: Commands,
) {
    for entity in render_query.iter() {
        commands.entity(entity).despawn();
    }

    if !should_render_tile_collision.get() {
        return;
    }

    for (collision, pos) in character_query {
        match collision.collider_type() {
            ColliderType::Cuboid(_) => todo!(),
            ColliderType::ConvexHull { .. } => todo!(),
            ColliderType::Capsule(capsule) => {
                let (edges, circles) = draw_capsule(
                    pos.0,
                    *capsule,
                    LineSettings {
                        color: KINEMATIC_COLLIDER_COLOR,
                        thickness: COLLIDER_LINE_THICKNESS,
                    },
                    scale.0,
                );

                edges.into_iter().for_each(|line| {
                    commands.spawn((CharacterCollisionRender, line));
                });

                circles.into_iter().for_each(|circle| {
                    commands.spawn((CharacterCollisionRender, circle));
                });
            }
        }
    }
}

//------ Tile Collision ------//

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
    tile_query: Query<(&Collider, &TilePosition), With<Tile>>,
    scale: Res<Scale>,
    mut commands: Commands,
) {
    for (collision, pos) in tile_query {
        match collision.collider_type() {
            ColliderType::Cuboid(cuboid) => {
                let half_extents = Vec3::new(
                    cuboid.half_extents.x,
                    cuboid.half_extents.y,
                    cuboid.half_extents.z,
                );

                draw_cuboid(
                    pos.0.as_vec3(),
                    half_extents,
                    LineSettings {
                        color: STATIC_COLLIDER_COLOR,
                        thickness: COLLIDER_LINE_THICKNESS,
                    },
                    scale.0,
                )
                .into_iter()
                .for_each(|line| {
                    commands.spawn((TileCollisionRender, line));
                });
            }
            ColliderType::ConvexHull { .. } => {}
            ColliderType::Capsule(_) => unreachable!("Tiles should not have capsule colliders"),
        }
    }
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
