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

use crate::{AppSystems, PausableSystems};
use crate::game::grid::coords::{TileCoords, WorldPosition};
use crate::game::grid::Grid;

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

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &mut WorldPosition)>,
    grid_query: Query<&Grid>,
) {
    for (controller, mut world_position) in &mut movement_query {
        if let Ok(grid) = grid_query.single() {
            let tile_map = grid.0.read().unwrap();

            let velocity = controller.intent * controller.max_speed;

            // Position we intend to move to
            let intended_position = world_position.0.0 + velocity * time.delta_secs();

            // Actual position we will move to
            let mut final_position = world_position.0.0;

            // Try X axis
            let test_x: TileCoords = Vec3::new(intended_position.x, world_position.0.0.y, world_position.0.0.z).into();
            let test_x_above: TileCoords = Vec3::new(intended_position.x, world_position.0.0.y + 1.0, world_position.0.0.z).into();

            if tile_map.contains_key(&test_x.0.into()) && !tile_map.contains_key(&test_x_above.0.into()) {
                final_position.x = intended_position.x;
            }
            else {
                // Clamp to current tile boundary on X axis
                let current_tile_x = world_position.0.0.x.round();
                let direction = (intended_position.x - world_position.0.0.x).signum();

                let boundary_x = current_tile_x + direction * (0.5 - TILE_BOUNDARY_SIZE);

                if direction > 0.0 {
                    final_position.x = intended_position.x.min(boundary_x);
                }
                else if direction < 0.0 {
                    final_position.x = intended_position.x.max(boundary_x);
                }
                else {
                    final_position.x = world_position.0.0.x;
                }
            }

            // Try Z axis
            let test_z: TileCoords = Vec3::new(final_position.x, world_position.0.0.y, intended_position.z).into();
            let test_z_above: TileCoords = Vec3::new(final_position.x, world_position.0.0.y + 1.0, intended_position.z).into();

            if tile_map.contains_key(&test_z.0.into()) && !tile_map.contains_key(&test_z_above.0.into()) {
                final_position.z = intended_position.z;
            }
            else {
                // Clamp to current tile boundary on Z axis
                let current_tile_z = world_position.0.0.z.round();
                let direction = (intended_position.z - world_position.0.0.z).signum();

                let boundary_z = current_tile_z + direction * (0.5 - TILE_BOUNDARY_SIZE);

                if direction > 0.0 {
                    final_position.z = intended_position.z.min(boundary_z);
                }
                else if direction < 0.0 {
                    final_position.z = intended_position.z.max(boundary_z);
                }
                else {
                    final_position.z = world_position.0.0.z;
                }
            }

            // Y axis can move freely (for jumping, etc.)
            final_position.y = intended_position.y;

            world_position.0.0 = final_position;
        }
    }
}