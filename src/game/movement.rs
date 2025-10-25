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
pub const STEP_UP_THRESHOLD: f32 = 0.1;

pub const NUM_MOVEMENT_STEPS: usize = 20;

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &mut WorldPosition)>,
    tile_query: Query<(Entity, &TileCollision)>,
    grid_query: Query<&Grid>,
) {
    for _ in 0..NUM_MOVEMENT_STEPS {
        for (controller, mut controller_position) in &mut movement_query {
            if let Ok(grid) = grid_query.single() {
                let tile_map = grid.0.read().unwrap();
                let velocity = controller.intent * controller.max_speed / NUM_MOVEMENT_STEPS as f32;

                let world_position = controller_position.0.0;


                // Position we intend to move to
                let intended_position = world_position + velocity * time.delta_secs();
                // Actual position we will move to
                let mut final_position = world_position;

                // Set height to collision height of current tile
                let current_tile = tile_map.get(&world_position.into());
                if let Some(tile) = current_tile {
                    for (entity, tile_collision) in tile_query.iter() {
                        if &entity == tile {
                            let collision_height = tile_collision.get_height(intended_position.x, intended_position.z) + world_position.y - 1.0;
                            final_position.y = collision_height;
                        }
                    }
                }

                //------ X Axis Movement ------//

                // Position we intend to walk onto
                let test_position: TileCoords = Vec3::new(
                    intended_position.x,
                    world_position.y,
                    world_position.z,
                )
                    .into();
                let test_tile = tile_map.get(&test_position.0.into());

                // Position above the tile we intend to walk onto
                let test_position_above: TileCoords = Vec3::new(
                    intended_position.x,
                    world_position.y + 1.0,
                    world_position.z,
                )
                    .into();
                let test_tile_above = tile_map.get(&test_position_above.0.into());

                let mut moved = false;

                // If there is a tile to walk onto, and nothing above it, move as normal
                if let Some(tile) = test_tile
                    && let None = test_tile_above
                {
                    for (entity, tile_collision) in tile_query.iter() {
                        if &entity == tile {
                            // Check collision
                            let test_height = tile_collision.get_height(intended_position.x, world_position.z) + test_position.y as f32 - 1.0;
                            if test_height <= world_position.y + STEP_UP_THRESHOLD {
                                final_position.x = intended_position.x;
                                final_position.y = test_height;
                                moved = true;
                            }
                        }
                    }
                } else if let Some(tile_above) = test_tile_above {
                    for (entity, tile_collision) in tile_query.iter() {
                        if &entity == tile_above {
                            // Check collision
                            let test_height = tile_collision.get_height(intended_position.x, world_position.z) + test_position_above.y as f32 - 1.0;
                            if test_height <= world_position.y + STEP_UP_THRESHOLD {
                                final_position.x = intended_position.x;
                                final_position.y = test_height;
                                moved = true;
                            }
                        }
                    }
                }

                // If we haven't moved yet, then move up to the boundary as far as we are able to
                if !moved {
                    // Clamp to current tile boundary on X axis
                    let current_tile_x = world_position.x.round();
                    let direction = (intended_position.x - world_position.x).signum();

                    let boundary = current_tile_x + direction * (0.5 - TILE_BOUNDARY_SIZE);

                    if direction > 0.0 {
                        final_position.x = intended_position.x.min(boundary);
                    } else if direction < 0.0 {
                        final_position.x = intended_position.x.max(boundary);
                    } else {
                        final_position.x = world_position.x;
                    }
                }

                //------ Z Axis Movement ------//

                // Position we intend to walk onto
                let test_position: TileCoords = Vec3::new(
                    final_position.x,
                    world_position.y,
                    intended_position.z,
                )
                    .into();
                let test_tile = tile_map.get(&test_position.0.into());

                // Position above the tile we intend to walk onto
                let test_position_above: TileCoords = Vec3::new(
                    final_position.x,
                    world_position.y + 1.0,
                    intended_position.z,
                )
                    .into();
                let test_tile_above = tile_map.get(&test_position_above.0.into());

                let mut moved = false;

                // If there is a tile to walk onto, and nothing above it, move as normal
                if let Some(tile) = test_tile
                    && let None = test_tile_above
                {
                    for (entity, tile_collision) in tile_query.iter() {
                        if &entity == tile {
                            // Check collision
                            let test_height = tile_collision.get_height(final_position.x, intended_position.z) + test_position.y as f32 - 1.0;
                            if test_height <= world_position.y + STEP_UP_THRESHOLD {
                                final_position.z = intended_position.z;
                                final_position.y = test_height;
                                moved = true;
                            }
                        }
                    }
                } else if let Some(tile_above) = test_tile_above {
                    for (entity, tile_collision) in tile_query.iter() {
                        if &entity == tile_above {
                            // Check collision
                            let test_height = tile_collision.get_height(final_position.x, intended_position.z) + test_position_above.y as f32 - 1.0;
                            if test_height <= world_position.y + STEP_UP_THRESHOLD {
                                final_position.z = intended_position.z;
                                final_position.y = test_height;
                                moved = true;
                            }
                        }
                    }
                }

                // If we haven't moved yet, then move up to the boundary as far as we are able to
                if !moved {
                    // Clamp to current tile boundary on X axis
                    let current_tile_z = world_position.z.round();
                    let direction = (intended_position.z - world_position.z).signum();

                    let boundary = current_tile_z + direction * (0.5 - TILE_BOUNDARY_SIZE);

                    if direction > 0.0 {
                        final_position.z = intended_position.z.min(boundary);
                    } else if direction < 0.0 {
                        final_position.z = intended_position.z.max(boundary);
                    } else {
                        final_position.z = world_position.z;
                    }
                }

                // Y axis can move freely (for jumping, etc.)
                //final_position.y = intended_position.y;

                controller_position.0.0 = final_position;
            }
        }
    }
}
