use crate::Scale;
use crate::game::grid::tile::{TILE_HEIGHT, TILE_WIDTH};
use bevy::prelude::*;
use std::cmp::Ordering;
use std::ops::Deref;

pub(super) const SCREEN_Z_SCALE: f32 = 2.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PreUpdate,
        (
            convert_world_to_screen_coords,
            convert_tile_to_screen_coords,
        ),
    );
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct WorldPosition(pub WorldCoords);
pub fn convert_world_to_screen_coords(
    scale: Res<Scale>,
    mut query: Query<(&WorldPosition, &mut Transform), Changed<WorldPosition>>,
) {
    for (world_position, mut transform) in query.iter_mut() {
        let screen_coords = ScreenCoords::from(&world_position.0);
        transform.translation = *screen_coords;
        transform.translation.x *= scale.0;
        transform.translation.y *= scale.0;
        // Offset to render in front of tiles
        transform.translation.z += SCREEN_Z_SCALE;
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct TilePosition(pub TileCoords);
pub fn convert_tile_to_screen_coords(
    mut query: Query<(&TilePosition, &mut Transform), Changed<TilePosition>>,
) {
    for (tile_position, mut transform) in query.iter_mut() {
        let screen_coords = ScreenCoords::from(&tile_position.0);
        transform.translation = *screen_coords;

        transform.translation.y -= TILE_HEIGHT as f32;
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Reflect)]
pub struct TileCoords(pub IVec3);
impl From<WorldCoords> for TileCoords {
    fn from(value: WorldCoords) -> Self {
        Self::from(value.0)
    }
}
impl From<&WorldCoords> for TileCoords {
    fn from(value: &WorldCoords) -> Self {
        Self(value.0.as_ivec3())
    }
}
impl From<IVec3> for TileCoords {
    fn from(value: IVec3) -> Self {
        TileCoords(value)
    }
}
impl From<Vec3> for TileCoords {
    fn from(value: Vec3) -> Self {
        // Use round() instead of as_ivec3() to get proper rounding
        TileCoords(IVec3::new(
            value.x.round() as i32,
            value.y.round() as i32,
            value.z.round() as i32,
        ))
    }
}
impl Deref for TileCoords {
    type Target = IVec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Ord for TileCoords {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.y.cmp(&other.y) {
            Ordering::Equal => match self.z.cmp(&other.z) {
                Ordering::Equal => self.x.cmp(&other.x),
                ordering => ordering,
            },
            ordering => ordering,
        }
    }
}
impl PartialOrd for TileCoords {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Clone, Reflect)]
pub struct WorldCoords(pub Vec3);
impl From<TileCoords> for WorldCoords {
    fn from(value: TileCoords) -> Self {
        Self::from(value.0)
    }
}
impl From<&TileCoords> for WorldCoords {
    fn from(value: &TileCoords) -> Self {
        Self(value.0.as_vec3())
    }
}
impl From<Vec3> for WorldCoords {
    fn from(value: Vec3) -> Self {
        WorldCoords(value)
    }
}
impl From<IVec3> for WorldCoords {
    fn from(value: IVec3) -> Self {
        WorldCoords(value.as_vec3())
    }
}
impl Deref for WorldCoords {
    type Target = Vec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct ScreenCoords(pub Vec3);
impl From<WorldCoords> for ScreenCoords {
    fn from(value: WorldCoords) -> Self {
        let screen_x = (value.x - value.z) * TILE_WIDTH as f32 / 2.0;
        let screen_y =
            ((value.y - 1.0) * TILE_HEIGHT as f32) - (value.x + value.z) * TILE_HEIGHT as f32 / 2.0;

        let screen_z = (value.x + value.z + (value.y)) * SCREEN_Z_SCALE;

        Vec3::new(screen_x, screen_y, screen_z).into()
    }
}
impl From<&WorldCoords> for ScreenCoords {
    fn from(value: &WorldCoords) -> Self {
        let screen_x = (value.x - value.z) * TILE_WIDTH as f32 / 2.0;
        let screen_y =
            ((value.y - 1.0) * TILE_HEIGHT as f32) - (value.x + value.z) * TILE_HEIGHT as f32 / 2.0;

        let screen_z = (value.x + value.z + (value.y)) * SCREEN_Z_SCALE;

        Vec3::new(screen_x, screen_y, screen_z).into()
    }
}
impl From<TileCoords> for ScreenCoords {
    fn from(value: TileCoords) -> Self {
        let screen_x = (value.x - value.z) * TILE_WIDTH / 2;
        let screen_y = (value.y * TILE_HEIGHT) - (value.x + value.z) * TILE_HEIGHT / 2;

        let screen_z = value.x as f32 + value.z as f32 + value.y as f32;

        Vec3::new(screen_x as f32, screen_y as f32, screen_z).into()
    }
}
impl From<&TileCoords> for ScreenCoords {
    fn from(value: &TileCoords) -> Self {
        let screen_x = (value.x - value.z) * TILE_WIDTH / 2;
        let screen_y = (value.y * TILE_HEIGHT) - (value.x + value.z) * TILE_HEIGHT / 2;

        let screen_z = value.x as f32 + value.z as f32 + value.y as f32;

        Vec3::new(screen_x as f32, screen_y as f32, screen_z).into()
    }
}
impl From<Vec3> for ScreenCoords {
    fn from(value: Vec3) -> Self {
        ScreenCoords(value)
    }
}
impl Deref for ScreenCoords {
    type Target = Vec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn rotate_movement_to_screen_space(movement: Vec3) -> Vec3 {
    let angle = -std::f32::consts::FRAC_PI_4;
    let rotation = Quat::from_rotation_y(angle);
    rotation * movement
}

pub fn rotate_screen_space_to_movement(screen_space: Vec3) -> Vec3 {
    let angle = std::f32::consts::FRAC_PI_4;
    let rotation = Quat::from_rotation_y(angle);
    rotation * screen_space
}
