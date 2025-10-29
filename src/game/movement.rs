//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use bevy::prelude::*;

use crate::game::grid::coords::{TileCoords, WorldPosition};
use crate::game::grid::{Grid, TileCollision};
use crate::game::object::{Collider, ColliderType};
use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (apply_movement)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec3,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec3::ZERO,
            max_speed: 3.5,
        }
    }
}

pub const TILE_BOUNDARY_SIZE: f32 = 0.02;
pub const STEP_UP_THRESHOLD: f32 = 0.3;

pub const GRAVITY: f32 = 4.5;

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &mut WorldPosition, &Collider)>,
    tile_query: Query<(Entity, &TileCollision)>,
    grid_query: Query<&Grid>,
) {
    for (controller, mut controller_position, collider) in &mut movement_query {
        if let Ok(grid) = grid_query.single() {
            let tile_map = grid.0.read().unwrap();

            let velocity = controller.intent * controller.max_speed;
            let velocity =
                if velocity.x.abs() < 0.01 && velocity.y.abs() < 0.01 && velocity.z.abs() < 0.01 {
                    Vec3::ZERO
                } else {
                    velocity
                };

            let world_position = controller_position.0.0;

            // Position we intend to move to
            let intended_center_position = world_position + velocity * time.delta_secs();

            // Get collider information
            let (collider_offset_x, collider_offset_z) = match collider.0 {
                ColliderType::Rectangle(size) => (size.x, size.z),
                ColliderType::Cylinder { radius, .. } => (radius, radius),
            };
            let direction_x = velocity.x.signum();
            let direction_z = velocity.z.signum();

            // Leading collider edge position
            let intended_collider_edge_position_x = world_position
                + velocity * Vec3::X * time.delta_secs()
                + Vec3::new(collider_offset_x, 0.0, 0.0) * direction_x;
            let intended_collider_edge_position_z = world_position
                + velocity * Vec3::Z * time.delta_secs()
                + Vec3::new(0.0, 0.0, collider_offset_z) * direction_z;

            // Actual position we will move to
            let mut final_position = world_position;

            // For X axis:
            let final_pos_x = check_axis_movement(
                grid,
                time.delta_secs(),
                world_position,
                intended_center_position,
                intended_collider_edge_position_x,
                collider_offset_x,
                direction_x,
                Vec3::X,
                tile_query,
            );

            // For Z axis:
            let final_pos_z = check_axis_movement(
                grid,
                time.delta_secs(),
                world_position,
                intended_center_position,
                intended_collider_edge_position_z,
                collider_offset_z,
                direction_z,
                Vec3::Z,
                tile_query,
            );

            final_position.x = final_pos_x.x;
            final_position.z = final_pos_z.z;

            final_position.y = final_pos_x.y.max(final_pos_z.y);

            let mut target_height = final_position.y;

            // Set height to collision height of current tile
            let current_tile = tile_map.get(&final_position.into());
            if let Some(tile) = current_tile {
                for (entity, tile_collision) in tile_query.iter() {
                    if &entity == tile {
                        let collision_height = tile_collision
                            .get_height(intended_center_position.x, intended_center_position.z)
                            + final_position.y
                            - 1.0;
                        target_height = collision_height;
                    }
                }
            }

            if target_height < final_position.y {
                if final_position.y - target_height > GRAVITY * time.delta_secs() {
                    final_position.y -= GRAVITY * time.delta_secs();
                } else {
                    final_position.y = target_height;
                }
            }

            controller_position.0.0 = final_position;
        }
    }
}

fn check_axis_movement(
    grid: &Grid,
    delta_secs: f32,
    world_position: Vec3,
    intended_center_position: Vec3,
    intended_collider_edge_position: Vec3,
    collider_offset: f32,
    direction: f32,
    axis_mask: Vec3,
    tile_query: Query<(Entity, &TileCollision)>,
) -> Vec3 {
    let mut final_position = world_position;

    let tile_map = grid.0.read().unwrap();

    // Position we intend to walk onto
    let test_position: TileCoords = Vec3::new(
        intended_center_position.x,
        world_position.y,
        intended_center_position.z,
    )
    .into();

    let test_collider_position: TileCoords = Vec3::new(
        intended_collider_edge_position.x,
        world_position.y,
        intended_collider_edge_position.z,
    )
    .into();
    let test_collider_tile = tile_map.get(&test_collider_position.0.into());

    // Position above the tile we intend to walk onto
    let test_position_above: TileCoords = Vec3::new(
        intended_center_position.x,
        world_position.y + 1.0,
        intended_center_position.z,
    )
    .into();
    let test_tile_above = tile_map.get(&test_position_above.0.into());

    let test_collider_position_above: TileCoords = Vec3::new(
        intended_collider_edge_position.x,
        world_position.y + 1.0,
        intended_collider_edge_position.z,
    )
    .into();
    let test_collider_tile_above = tile_map.get(&test_collider_position_above.0.into());

    let mut target_height = world_position.y;

    let moved = match (
        test_collider_tile,
        test_tile_above,
        test_collider_tile_above,
    ) {
        // There is something for us to run into or to walk onto
        (Some(_), Some(tile_above), _) => {
            if let Ok((_, tile_collision)) = tile_query.get(*tile_above) {
                // Check collision
                let test_height = tile_collision
                    .get_height(intended_center_position.x, intended_center_position.z)
                    + test_position_above.y as f32
                    - 1.0;
                if test_height.clamp(0.0, 1.0) <= world_position.y + STEP_UP_THRESHOLD {
                    final_position.x = intended_center_position.x;
                    final_position.z = intended_center_position.z;
                    target_height = test_height;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        // We are approaching something to run into or to walk onto but haven't reached it yet
        (Some(_), _, Some(collider_tile_above)) => {
            if let Ok((_, tile_collision)) = tile_query.get(*collider_tile_above) {
                // Check collision
                let test_height = tile_collision.get_height(
                    intended_collider_edge_position.x,
                    intended_collider_edge_position.z,
                ) + test_position_above.y as f32
                    - 1.0;

                if test_height <= world_position.y + STEP_UP_THRESHOLD {
                    final_position.x = intended_center_position.x;
                    final_position.z = intended_center_position.z;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        // There is nothing to run into at our level, and there is ground to walk onto
        (Some(collider_tile), None, None) => {
            if let Ok((_, tile_collision)) = tile_query.get(*collider_tile) {
                // Check collision
                let test_height = tile_collision
                    .get_height(intended_center_position.x, intended_center_position.z)
                    + test_position.y as f32
                    - 1.0;
                if test_height <= world_position.y + STEP_UP_THRESHOLD {
                    final_position.x = intended_center_position.x;
                    final_position.z = intended_center_position.z;
                    target_height = test_height;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        // All other cases
        (_, _, _) => false,
    };

    if !moved {
        // Clamp to current tile boundary
        let current_tile_coord = (world_position * axis_mask).dot(axis_mask).round();
        let boundary = current_tile_coord + direction * (0.5 - TILE_BOUNDARY_SIZE)
            - collider_offset * direction;
        let intended_coord = (intended_center_position * axis_mask).dot(axis_mask);

        let clamped_coord = if direction > 0.0 {
            intended_coord.min(boundary)
        } else if direction < 0.0 {
            intended_coord.max(boundary)
        } else {
            (world_position * axis_mask).dot(axis_mask)
        };

        final_position += axis_mask * (clamped_coord - (world_position * axis_mask).dot(axis_mask));
    }

    if world_position.y - target_height > GRAVITY * delta_secs {
        final_position.y = world_position.y - GRAVITY * delta_secs;
    } else {
        final_position.y = target_height;
    }

    final_position
}
