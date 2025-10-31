use crate::game::grid::coords::WorldCoords;
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Reflect)]
pub enum PhysicsData {
    Static,
    Kinematic { velocity: Vec3 },
}

impl PhysicsData {
    pub fn kinematic(velocity: Vec3) -> Self {
        Self::Kinematic { velocity }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Collider {
    collider_type: ColliderType,
    position: WorldCoords,
}

impl Collider {
    pub fn aabb(size: Vec3, position: WorldCoords) -> Self {
        Self {
            collider_type: ColliderType::AABB(size),
            position,
        }
    }

    pub fn capsule(radius: f32, height: f32, position: WorldCoords) -> Self {
        Self {
            collider_type: ColliderType::Capsule { radius, height },
            position,
        }
    }

    pub fn hull(points: Vec<Vec3>, position: WorldCoords) -> Self {
        Self {
            collider_type: ColliderType::Hull { points },
            position,
        }
    }

    pub fn get(&self) -> &ColliderType {
        &self.collider_type
    }

    pub fn check_collision(&self, other: &Self) -> Option<Collision> {
        let rough_collision = self.check_collision_rough(other)?;

        use ColliderType as C;
        match (&self.collider_type, &other.collider_type) {
            (C::AABB(size), C::AABB(other_size)) => Self::check_collision_aabb(size, &self.position, other_size, &other.position),
            (
                C::Capsule { radius, height },
                C::Capsule {
                    radius: other_radius,
                    height: other_height,
                },
            ) => Self::check_collision_capsule(*radius, *height, &self.position, *other_radius, *other_height, &other.position),
            (
                C::Hull { points },
                C::Hull {
                    points: other_points,
                },
            ) => Self::check_collision_hull(points, &self.position, other_points, &other.position),

            (
                C::AABB(size),
                C::Capsule {
                    radius: other_radius,
                    height: other_height,
                },
            ) => Self::check_collision_aabb_capsule(size, &self.position, *other_radius, *other_height, &other.position),
            (C::Capsule { radius, height }, C::AABB(other_size)) => {
                Self::check_collision_aabb_capsule(other_size, &other.position, *radius, *height, &self.position)
            }

            (
                C::AABB(size),
                C::Hull {
                    points: other_points,
                },
            ) => Self::check_collision_aabb_hull(size, &self.position, other_points, &other.position),
            (C::Hull { points }, C::AABB(other_size)) => {
                Self::check_collision_aabb_hull(other_size, &other.position, points, &self.position)
            }

            (
                C::Capsule { radius, height },
                C::Hull {
                    points: other_points,
                },
            ) => Self::check_collision_capsule_hull(*radius, *height, &self.position, other_points, &other.position),
            (
                C::Hull { points },
                C::Capsule {
                    radius: other_radius,
                    height: other_height,
                },
            ) => Self::check_collision_capsule_hull(*other_radius, *other_height, &other.position, points, &self.position),
        };

        // Temporarily just return rough collision
        Some(rough_collision)
    }

    fn check_collision_rough(&self, other: &Self) -> Option<Collision> {
        let dist = (*self.position - *other.position).length();

        if dist > self.get_largest_radius() + other.get_largest_radius() {
            Some(Collision {
                position: WorldCoords((*self.position + *other.position) / 2.0),
                normal: (*self.position - *other.position).normalize(),
                depth: dist - self.get_largest_radius() - other.get_largest_radius(),
            })
        } else { None }
    }

    fn get_largest_radius(&self) -> f32 {
        match &self.collider_type {
            ColliderType::AABB(size) => size.x.max(size.y).max(size.z),
            ColliderType::Capsule { height, .. } => height / 2.0,
            ColliderType::Hull { points } => {
                let mut largest_radius = 0.0;
                for point in points {
                    let distance = point.length();
                    if distance > largest_radius {
                        largest_radius = distance;
                    }
                }
                largest_radius
            }
        }
    }

    fn check_collision_aabb(
        size: &Vec3,
        position: &WorldCoords,
        other_size: &Vec3,
        other_position: &WorldCoords,
    ) -> Option<Collision> {
        let min_pos = **position + size / 2.0;
        let max_pos = **position + size / 2.0;

        let min_other_pos = **other_position + other_size / 2.0;
        let max_other_pos = **other_position + other_size / 2.0;

        // If there is a collision
        if min_pos.x <= max_other_pos.x
            && max_pos.x >= min_other_pos.x
            && min_pos.y <= max_other_pos.y
            && max_pos.y >= min_other_pos.y
            && min_pos.z <= max_other_pos.z
            && max_pos.z >= min_other_pos.z
        {
            // Get the overlap in each axis
            let total_dist_x = (max_pos.x - min_other_pos.x).abs();
            let total_dist_y = (max_pos.y - min_other_pos.y).abs();
            let total_dist_z = (max_pos.z - min_other_pos.z).abs();

            let combined_size_x = size.x + other_size.x;
            let combined_size_y = size.y + other_size.y;
            let combined_size_z = size.z + other_size.z;

            let overlaps = Vec3::new(
                combined_size_x - total_dist_x,
                combined_size_y - total_dist_y,
                combined_size_z - total_dist_z,
            );

            // Find the largest overlap
            let position = ((**position + **other_position) / 2.0).into();
            let depth = overlaps.max_element();
            let normal = if depth == overlaps.x {
                Vec3::X
            } else if depth == overlaps.y {
                Vec3::Y
            } else {
                Vec3::Z
            };

            Some(Collision {
                position,
                normal,
                depth,
            })
        } else { None }
    }

    fn check_collision_capsule(
        radius: f32,
        height: f32,
        position: &WorldCoords,
        other_radius: f32,
        other_height: f32,
        other_position: &WorldCoords,
    ) -> Option<Collision> {
        None
    }

    fn check_collision_hull(
        points: &[Vec3],
        position: &WorldCoords,
        other_points: &[Vec3],
        other_position: &WorldCoords,
    ) -> Option<Collision> {
        None
    }

    fn check_collision_aabb_capsule(
        aabb_size: &Vec3,
        aabb_position: &WorldCoords,
        capsule_radius: f32,
        capsule_height: f32,
        capsule_position: &WorldCoords,
    ) -> Option<Collision> {
        None
    }

    fn check_collision_aabb_hull(
        aabb_size: &Vec3,
        aabb_position: &WorldCoords,
        hull_points: &[Vec3],
        hull__position: &WorldCoords,
    ) -> Option<Collision> {
        None
    }

    fn check_collision_capsule_hull(
        capsule_radius: f32,
        capsule_height: f32,
        capsule_position: &WorldCoords,
        hull_points: &[Vec3],
        hull__position: &WorldCoords,
    ) -> Option<Collision> {
        None
    }
}

#[derive(Debug, Clone, Reflect)]
pub enum ColliderType {
    AABB(Vec3),
    Capsule { radius: f32, height: f32 },
    Hull { points: Vec<Vec3> },
}

pub struct Collision {
    position: WorldCoords,
    normal: Vec3,
    depth: f32,
}
