use crate::debug_options::options::{AttackCollisionRes, CharacterCollisionRes, TileCollisionRes};
use crate::debug_options::render::helpers::*;
use crate::debug_options::render::palette::*;
use bevy::prelude::*;
use common::dev_tools::DebugState;
use common::{GameState, Scale, TilePosition, WorldCoords, WorldPosition, marker};
use physics::{Collider, ColliderData};
use runtime::characters::Character;
use runtime::debug::{AttackHitbox, Tile};

pub(super) fn plugin(app: &mut App) {
    // Tile collision uses retained state for rendering because tile collision does not move
    // Character collision does move, and so it uses immediate mode rendering

    app.add_systems(
        Update,
        (
            update_character_collision_render,
            update_attack_collision_render,
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
        render_collider(
            collision,
            pos.0,
            &scale,
            LineSettings {
                color: KINEMATIC_COLLIDER_COLOR,
                thickness: COLLIDER_LINE_THICKNESS,
            },
            CharacterCollisionRender,
            commands.reborrow(),
        );
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
    for (collider, pos) in tile_query {
        render_collider(
            collider,
            pos.0.into(),
            &scale,
            LineSettings {
                color: STATIC_COLLIDER_COLOR,
                thickness: COLLIDER_LINE_THICKNESS,
            },
            TileCollisionRender,
            commands.reborrow(),
        )
    }
}

fn tile_collision_bundle() -> impl Bundle {
    (TileCollisionRender, DespawnOnExit(GameState::Gameplay))
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

//------ Attack Collision ------//

marker!(AttackCollisionRender);

fn update_attack_collision_render(
    attack_query: Query<(&Collider, &WorldPosition), With<AttackHitbox>>,
    render_query: Query<Entity, With<AttackCollisionRender>>,
    scale: Res<Scale>,
    should_render_tile_collision: Res<AttackCollisionRes>,
    mut commands: Commands,
) {
    for entity in render_query.iter() {
        commands.entity(entity).despawn();
    }

    if !should_render_tile_collision.get() {
        return;
    }

    for (collision, pos) in attack_query {
        render_collider(
            collision,
            pos.0,
            &scale,
            LineSettings {
                color: ATTACK_COLLIDER_COLOR,
                thickness: ATTACK_COLLIDER_LINE_THICKNESS,
            },
            AttackCollisionRender,
            commands.reborrow(),
        );
    }
}

//------ Helpers ------//

fn render_collider(
    collision: &Collider,
    pos: WorldCoords,
    scale: &Scale,
    settings: LineSettings,
    marker: impl Component + Copy,
    mut commands: Commands,
) {
    match collision.collider_type() {
        ColliderData::Cuboid(cuboid) => {
            let half_extents = Vec3::new(
                cuboid.half_extents.x,
                cuboid.half_extents.y,
                cuboid.half_extents.z,
            );

            draw_cuboid(
                pos.0.into(),
                half_extents,
                LineSettings {
                    color: STATIC_COLLIDER_COLOR,
                    thickness: COLLIDER_LINE_THICKNESS,
                },
                scale.0,
            )
            .into_iter()
            .for_each(|line| {
                commands.spawn((tile_collision_bundle(), line));
            });
        }
        ColliderData::ConvexHull {
            vertices, indices, ..
        } => {
            draw_convex_hull(
                pos.0.into(),
                vertices,
                indices,
                LineSettings {
                    color: CONVEX_HULL_COLOR,
                    thickness: COLLIDER_LINE_THICKNESS,
                },
                scale.0,
            )
            .into_iter()
            .for_each(|line| {
                commands.spawn((tile_collision_bundle(), line));
            });
        }
        ColliderData::Capsule(capsule) => {
            let lines = draw_capsule(pos, *capsule, settings, scale.0);

            lines.into_iter().for_each(|line| {
                commands.spawn((marker, line));
            });
        }
    }
}
