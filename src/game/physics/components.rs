use crate::game::grid::coords::{WorldCoords, WorldPosition};
use crate::{AppSystems, PausableSystems};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, update_collider_position);
}

#[derive(Component, Debug, Clone, Reflect)]
pub enum PhysicsData {
    Static,
    Kinematic {
        displacement: Vec3,
        grounded: bool,
        // Used for coyote time
        time_since_grounded: f32,
    },
}

impl PhysicsData {
    pub fn kinematic(velocity: Vec3) -> Self {
        Self::Kinematic {
            displacement: velocity,
            grounded: false,
            time_since_grounded: f32::INFINITY,
        }
    }
}

fn update_collider_position(query: Query<(&mut Collider, &WorldPosition)>) {
    for (mut collider, world_position) in query {
        collider.position = world_position.0.clone();
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Collider {
    collider_type: ColliderType,
    position: WorldCoords,
}

impl Collider {
    pub fn aabb(size: Vec3, position: impl Into<WorldCoords>) -> Self {
        let mut position = position.into();
        position.0.y += size.y / 2.0;
        Self {
            collider_type: ColliderType::AABB(size),
            position,
        }
    }

    pub fn sphere(radius: f32, position: impl Into<WorldCoords>) -> Self {
        Self::capsule(radius, radius, position)
    }

    pub fn capsule(radius: f32, height: f32, position: impl Into<WorldCoords>) -> Self {
        let mut position = position.into();
        position.0.y += height / 2.0;
        Self {
            collider_type: ColliderType::Capsule { radius, height },
            position,
        }
    }

    pub fn hull(mut points: Vec<Vec3>, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        points.dedup();
        Self {
            collider_type: ColliderType::Hull { points },
            position,
        }
    }

    pub fn get(&self) -> &ColliderType {
        &self.collider_type
    }

    pub fn check_collision(&self, other: &Self) -> Option<Collision> {
        use ColliderType as C;
        match (&self.collider_type, &other.collider_type) {
            //self.check_collision_rough(other)?;
            (C::AABB(size), C::AABB(other_size)) => {
                Self::check_collision_aabb(size, &self.position, other_size, &other.position)
            }
            (
                C::Capsule { radius, height },
                C::Capsule {
                    radius: other_radius,
                    height: other_height,
                },
            ) => Self::check_collision_capsule(
                *radius,
                *height,
                &self.position,
                *other_radius,
                *other_height,
                &other.position,
            ),
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
            ) => Self::check_collision_aabb_capsule(
                size,
                &self.position,
                *other_radius,
                *other_height,
                &other.position,
                false,
            ),
            (C::Capsule { radius, height }, C::AABB(other_size)) => {
                Self::check_collision_aabb_capsule(
                    other_size,
                    &other.position,
                    *radius,
                    *height,
                    &self.position,
                    true,
                )
            }

            (
                C::AABB(size),
                C::Hull {
                    points: other_points,
                },
            ) => Self::check_collision_aabb_hull(
                size,
                &self.position,
                other_points,
                &other.position,
                false,
            ),
            (C::Hull { points }, C::AABB(other_size)) => Self::check_collision_aabb_hull(
                other_size,
                &other.position,
                points,
                &self.position,
                true,
            ),

            (
                C::Capsule { radius, height },
                C::Hull {
                    points: other_points,
                },
            ) => Self::check_collision_capsule_hull(
                *radius,
                *height,
                &self.position,
                other_points,
                &other.position,
                false,
            ),
            (
                C::Hull { points },
                C::Capsule {
                    radius: other_radius,
                    height: other_height,
                },
            ) => Self::check_collision_capsule_hull(
                *other_radius,
                *other_height,
                &other.position,
                points,
                &self.position,
                true,
            ),
        }
    }

    //------ Rough (spherical) collision ------//

    fn check_collision_rough(&self, other: &Self) -> Option<Collision> {
        let dist = (*self.position - *other.position).length();

        if dist < self.get_largest_radius() + other.get_largest_radius() {
            Some(Collision {
                position: WorldCoords((*self.position + *other.position) / 2.0),
                normal: -(*self.position - *other.position).normalize(),
                depth: dist - self.get_largest_radius() - other.get_largest_radius(),
            })
        } else {
            None
        }
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

    //------ AABB-AABB collision ------//

    fn check_collision_aabb(
        size: &Vec3,
        position: &WorldCoords,
        other_size: &Vec3,
        other_position: &WorldCoords,
    ) -> Option<Collision> {
        let min_pos = **position - size / 2.0;
        let max_pos = **position + size / 2.0;

        let min_other_pos = **other_position - other_size / 2.0;
        let max_other_pos = **other_position + other_size / 2.0;

        // If there is a collision
        if min_pos.x <= max_other_pos.x
            && max_pos.x >= min_other_pos.x
            && min_pos.y <= max_other_pos.y
            && max_pos.y >= min_other_pos.y
            && min_pos.z <= max_other_pos.z
            && max_pos.z >= min_other_pos.z
        {
            // Calculate the center-to-center vector
            let center_diff = **position - **other_position;

            // Get the combined half-extents
            let combined_half_extents = (size + other_size) / 2.0;

            // Calculate overlap on each axis
            // Overlap = combined size - distance between centers
            let overlaps = Vec3::new(
                combined_half_extents.x - center_diff.x.abs(),
                combined_half_extents.y - center_diff.y.abs(),
                combined_half_extents.z - center_diff.z.abs(),
            );

            // Find the axis with the smallest overlap (minimum penetration)
            let min_overlap = overlaps.min_element();

            // Determine collision normal based on the smallest overlap axis
            let (normal, depth) = if min_overlap == overlaps.x {
                (Vec3::X * center_diff.x.signum(), overlaps.x)
            } else if min_overlap == overlaps.y {
                (Vec3::Y * center_diff.y.signum(), overlaps.y)
            } else {
                (Vec3::Z * center_diff.z.signum(), overlaps.z)
            };

            Some(Collision {
                position: ((**position + **other_position) / 2.0).into(),
                normal,
                depth,
            })
        } else {
            None
        }
    }

    //------ Capsule-Capsule collision ------//

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

    //------ Hull-Hull collision ------//

    fn check_collision_hull(
        points: &[Vec3],
        position: &WorldCoords,
        other_points: &[Vec3],
        other_position: &WorldCoords,
    ) -> Option<Collision> {
        None
    }

    //------ AABB-Capsule collision ------//

    fn check_collision_aabb_capsule(
        aabb_size: &Vec3,
        aabb_position: &WorldCoords,
        capsule_radius: f32,
        capsule_height: f32,
        capsule_position: &WorldCoords,
        invert_normal: bool,
    ) -> Option<Collision> {
        let min_aabb_pos = **aabb_position - aabb_size / 2.0;
        let max_aabb_pos = **aabb_position + aabb_size / 2.0;

        let capsule_segment_length = capsule_height - capsule_radius * 2.0;
        let capsule_segment_bottom =
            **capsule_position - Vec3::new(0.0, capsule_segment_length / 2.0, 0.0);
        let capsule_segment_top =
            **capsule_position + Vec3::new(0.0, capsule_segment_length / 2.0, 0.0);

        // If the segment has overlap with the AABB along the y axis
        let mut collision: Option<Collision> = if min_aabb_pos.y < capsule_segment_top.y
            && max_aabb_pos.y > capsule_segment_bottom.y
        {
            // If the capsule segment is inside the AABB
            if min_aabb_pos.x < capsule_segment_top.x
                && capsule_segment_top.x < max_aabb_pos.x
                && min_aabb_pos.z < capsule_segment_top.z
                && capsule_segment_top.z < max_aabb_pos.z
            {
                // Calculate center-to-center distance
                let center_to_center = **capsule_position - **aabb_position;

                // Find the closest edge to the capsule segment
                let mut dist_to_pos = (max_aabb_pos - center_to_center).abs();
                let mut dist_to_neg = (center_to_center - min_aabb_pos).abs();

                dist_to_pos.y += capsule_radius;
                dist_to_neg.y += capsule_radius;

                let min_position_pos = dist_to_pos.min_position();
                let min_position_neg = dist_to_neg.min_position();

                let min_dist_pos = dist_to_pos.min_element();
                let min_dist_neg = dist_to_neg.min_element();

                let depth = min_dist_pos.min(min_dist_neg);

                let min_index = if min_dist_pos <= min_dist_neg {
                    min_position_pos
                } else {
                    min_position_neg + 3
                };

                // Depth vectors are offset by the other dim from the normal by design
                let (position, normal) = match min_index {
                    0 => {
                        let position = aabb_position.0 + max_aabb_pos - Vec3::new(0.0, 0.0, depth);
                        (position, Vec3::X)
                    }
                    1 => {
                        let position = aabb_position.0 + max_aabb_pos
                            - Vec3::new(capsule_position.x, depth, capsule_position.z);
                        (position, Vec3::Y)
                    }
                    2 => {
                        let position = aabb_position.0 + max_aabb_pos - Vec3::new(depth, 0.0, 0.0);
                        (position, Vec3::Z)
                    }
                    3 => {
                        let position = aabb_position.0 + min_aabb_pos + Vec3::new(0.0, 0.0, depth);
                        (position, Vec3::NEG_X)
                    }
                    4 => {
                        let position = aabb_position.0
                            + min_aabb_pos
                            + Vec3::new(capsule_position.x, depth, capsule_position.z);
                        (position, Vec3::Y)
                    }
                    5 => {
                        let position = aabb_position.0 + min_aabb_pos + Vec3::new(depth, 0.0, 0.0);
                        (position, Vec3::NEG_Z)
                    }
                    _ => unreachable!(),
                };

                let collision = Collision {
                    position: position.into(),
                    normal,
                    depth,
                };
                println!("Collision: {collision:?}");
                Some(collision)
            }
            // If the capsule segment is outside the AABB on one axis,
            // but closer than the capsule radius

            // If the capsule segment is outside the AABB on both axes,
            // but closer than the capsule radius
            else {
                None
            }
        }
        // If the segment is above or below the AABB, but closer than the capsule radius
        else {
            None
        };

        if invert_normal && let Some(ref mut collision) = collision {
            collision.normal *= -1.0;
        };

        collision
    }

    //------ AABB-Hull collision ------//

    fn check_collision_aabb_hull(
        aabb_size: &Vec3,
        aabb_position: &WorldCoords,
        hull_points: &[Vec3],
        hull_position: &WorldCoords,
        invert_normal: bool,
    ) -> Option<Collision> {
        None
    }

    //------ Capsule-Hull collision ------//

    fn check_collision_capsule_hull(
        capsule_radius: f32,
        capsule_height: f32,
        capsule_position: &WorldCoords,
        hull_points: &[Vec3],
        hull_position: &WorldCoords,
        invert_normal: bool,
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

#[derive(Debug, Clone)]
pub struct Collision {
    pub position: WorldCoords,
    pub normal: Vec3,
    pub depth: f32,
}
