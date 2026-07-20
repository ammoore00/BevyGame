use bevy::math::ops::sin_cos;
use crate::WorldCoords;
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum Facing {
    NorthWest = 0,
    West = 1,
    #[default]
    SouthWest = 2,
    South = 3,
    SouthEast = 4,
    East = 5,
    NorthEast = 6,
    North = 7,
}
impl Facing {
    pub fn angle(&self) -> f32 {
        let index = *self as usize;

        // Rotate so that Z is forward and X is sideways
        let total_offset = std::f32::consts::FRAC_PI_2;

        // Calculate the angle corresponding to the index.
        // index * (PI/4) gives the angle relative to our offset.
        (index as f32 * std::f32::consts::FRAC_PI_4) - total_offset
    }
}
impl From<usize> for Facing {
    fn from(index: usize) -> Self {
        match index {
            0 => Self::NorthWest,
            1 => Self::West,
            2 => Self::SouthWest,
            3 => Self::South,
            4 => Self::SouthEast,
            5 => Self::East,
            6 => Self::NorthEast,
            7 => Self::North,
            _ => unreachable!(),
        }
    }
}
impl From<Vec2> for Facing {
    fn from(vec: Vec2) -> Self {
        // Calculate angle in radians (-PI to PI)
        // Note: atan2(z, x) where x is "forward" and z is "right"
        let angle = vec.x.atan2(vec.y);

        // Convert to 0-8 range, where each direction occupies 45 degrees (PI/4 radians)
        // Add PI to shift range from [-PI, PI] to [0, 2*PI]
        // Add PI/8 to center the divisions on the cardinal directions
        // Add 3PI/2 to rotate divisions to align with sprite sheets
        // Divide by PI/4 (45 degrees) to get 0-8 range
        let direction_index = ((angle
            + std::f32::consts::PI
            + std::f32::consts::FRAC_PI_8
            + std::f32::consts::FRAC_PI_2 * 3.0)
            / std::f32::consts::FRAC_PI_4)
            .floor() as i32
            % 8;

        Self::from(direction_index as usize)
    }
}
impl From<Facing> for Vec2 {
    fn from(value: Facing) -> Self {
        let (sin, cos) = sin_cos(value.angle());
        Vec2::new(sin, cos)
    }
}
impl From<Facing> for Vec3 {
    fn from(value: Facing) -> Self {
        Vec3::new(Vec2::from(value).x, 0.0, Vec2::from(value).y)
    }
}

pub fn offset_position_to_facing(pos: WorldCoords, offset: WorldCoords, facing: Facing) -> WorldCoords {
    WorldCoords::from(*pos + offset.rotate_y(facing.angle()))
}