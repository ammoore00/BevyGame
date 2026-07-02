use assets::resource::TileFacing;
use bevy::prelude::*;
use common::WorldCoords;
use physics::Collider;

pub(crate) fn full() -> impl Fn(WorldCoords) -> Collider {
    cuboid(Vec3::splat(0.5))
}

pub(crate) fn slope_45(facing: TileFacing) -> impl Fn(WorldCoords) -> Collider {
    slope(0.0, 1.0, facing)
}

pub(crate) fn cuboid(size: Vec3) -> impl Fn(WorldCoords) -> Collider {
    move |pos| Collider::cuboid(size, pos)
}

pub(crate) fn slope(
    lower_height: f32,
    upper_height: f32,
    facing: TileFacing,
) -> impl Fn(WorldCoords) -> Collider {
    move |pos| {
        let points = [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.0, lower_height, 0.0],
            [0.0, lower_height, 1.0],
            [1.0, upper_height, 1.0],
            [1.0, upper_height, 0.0],
        ]
        .iter()
        .map(|point| facing.rotate_point(Vec3::from(*point) - Vec3::splat(0.5)))
        .collect::<Vec<_>>();

        Collider::convex_hull(&points, *pos)
    }
}

pub(crate) fn convex_hull(points: &[Vec3]) -> impl Fn(WorldCoords) -> Collider {
    move |pos| Collider::convex_hull(&Vec::from(points), pos)
}
