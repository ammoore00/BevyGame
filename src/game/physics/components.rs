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
            collider_type: ColliderType::Aabb(Aabb(size)),
            position,
        }
    }

    pub fn sphere(radius: f32, position: impl Into<WorldCoords>) -> Self {
        Self::capsule(Vec3::ZERO, Vec3::ZERO, radius, position.into())
    }

    pub fn vertical_capsule(
        radius: f32,
        mut height: f32,
        position: impl Into<WorldCoords>,
    ) -> Self {
        if height < radius * 2.0 {
            height = radius * 2.0;
        }

        height -= radius * 2.0;

        let mut position = position.into();
        position.0.y += radius;

        Self::capsule(
            Vec3::Y * radius,
            Vec3::Y * radius + Vec3::Y * height,
            radius,
            position,
        )
    }

    pub fn capsule(start: Vec3, end: Vec3, radius: f32, position: impl Into<WorldCoords>) -> Self {
        Self {
            collider_type: ColliderType::Capsule(Capsule { start, end, radius }),
            position: position.into(),
        }
    }

    pub fn hull(mut points: Vec<Vec3>, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        points.dedup();
        Self {
            collider_type: ColliderType::Hull(Hull(points)),
            position,
        }
    }

    pub fn get(&self) -> &ColliderType {
        &self.collider_type
    }

    pub fn check_collision(&self, other: &Self) -> Option<Collision> {
        self.check_collision_rough(other)?;

        use ColliderType as C;
        match (&self.collider_type, &other.collider_type) {
            // Symmetric colliders
            (C::Aabb(aabb), C::Aabb(other_aabb)) => {
                check_collision_aabb(aabb, &self.position, other_aabb, &other.position)
            }
            (C::Capsule(capsule), C::Capsule(other_capsule)) => {
                check_collision_capsule(capsule, &self.position, other_capsule, &other.position)
            }
            (C::Hull(hull), C::Hull(other_hull)) => {
                check_collision_hull(hull, &self.position, other_hull, &other.position)
            }

            // AABB-Capsule
            (C::Aabb(aabb), C::Capsule(capsule)) => {
                check_collision_aabb_capsule(aabb, &self.position, capsule, &other.position)
            }
            (C::Capsule(capsule), C::Aabb(aabb)) => {
                check_collision_capsule_aabb(capsule, &self.position, aabb, &other.position)
            }

            // AABB-Hull
            (C::Aabb(aabb), C::Hull(hull)) => {
                check_collision_aabb_hull(aabb, &self.position, hull, &other.position, false)
            }
            (C::Hull(hull), C::Aabb(aabb)) => {
                check_collision_aabb_hull(aabb, &other.position, hull, &self.position, true)
            }

            // Capsule-Hull
            (C::Capsule(capsule), C::Hull(hull)) => {
                check_collision_capsule_hull(capsule, &self.position, hull, &other.position, false)
            }
            (C::Hull(hull), C::Capsule(capsule)) => {
                check_collision_capsule_hull(capsule, &self.position, hull, &other.position, true)
            }
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
            ColliderType::Aabb(aabb) => aabb.0.x.max(aabb.0.y).max(aabb.0.z),
            ColliderType::Capsule(capsule) => {
                (capsule.start - capsule.end).length() / 2.0 + capsule.radius
            }
            ColliderType::Hull(hull) => {
                let points = &hull.0;
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
}

//------ AABB-AABB collision ------//

fn check_collision_aabb(
    aabb: &Aabb,
    position: &WorldCoords,
    other_aabb: &Aabb,
    other_position: &WorldCoords,
) -> Option<Collision> {
    let min_pos = **position - aabb.0 / 2.0;
    let max_pos = **position + aabb.0 / 2.0;

    let min_other_pos = **other_position - other_aabb.0 / 2.0;
    let max_other_pos = **other_position + other_aabb.0 / 2.0;

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
        let combined_half_extents = (aabb.0 + other_aabb.0) / 2.0;

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
    capsule: &Capsule,
    position: &WorldCoords,
    other_capsule: &Capsule,
    other_position: &WorldCoords,
) -> Option<Collision> {
    None
}

//------ Hull-Hull collision ------//

fn check_collision_hull(
    hull: &Hull,
    position: &WorldCoords,
    other_hull: &Hull,
    other_position: &WorldCoords,
) -> Option<Collision> {
    None
}

//------ AABB-Capsule collision ------//

fn check_collision_aabb_capsule(
    aabb: &Aabb,
    aabb_position: &WorldCoords,
    other_capsule: &Capsule,
    other_capsule_position: &WorldCoords,
) -> Option<Collision> {
    let aabb_min = **aabb_position - aabb.0 / 2.0;
    let aabb_max = **aabb_position + aabb.0 / 2.0;

    let closest_point_capsule = closest_point_capsule_aabb(
        other_capsule.start + other_capsule_position.0,
        other_capsule.end + other_capsule_position.0,
        aabb_min,
        aabb_max,
    );
    let closest_point_aabb = closest_point_on_aabb(closest_point_capsule, aabb_min, aabb_max);
    let distance = (closest_point_capsule - closest_point_aabb).length();

    if distance <= other_capsule.radius {
        let depth = other_capsule.radius - distance;

        let aabb_center = (aabb_min + aabb_max) * 0.5;
        let aabb_half_size = (aabb_max - aabb_min) * 0.5;
        let local = closest_point_aabb - aabb_center;

        let dx = aabb_half_size.x - local.x.abs();
        let dy = aabb_half_size.y - local.y.abs();
        let dz = aabb_half_size.z - local.z.abs();

        // Smallest penetration axis determines face
        let normal = if dx < dy && dx < dz {
            Vec3::new(local.x.signum(), 0.0, 0.0)
        } else if dy < dz {
            Vec3::new(0.0, local.y.signum(), 0.0)
        } else {
            Vec3::new(0.0, 0.0, local.z.signum())
        };

        let position = closest_point_capsule + normal * depth;

        Some(Collision {
            position: position.into(),
            normal,
            depth,
        })
    } else {
        None
    }
}

fn check_collision_capsule_aabb(
    capsule: &Capsule,
    capsule_position: &WorldCoords,
    other_aabb: &Aabb,
    other_aabb_position: &WorldCoords,
) -> Option<Collision> {
    let aabb_min = **other_aabb_position - other_aabb.0 / 2.0;
    let aabb_max = **other_aabb_position + other_aabb.0 / 2.0;

    let closest_point_capsule = closest_point_capsule_aabb(
        capsule.start + capsule_position.0,
        capsule.end + capsule_position.0,
        aabb_min,
        aabb_max,
    );
    let closest_point_aabb = closest_point_on_aabb(closest_point_capsule, aabb_min, aabb_max);
    let distance = (closest_point_capsule - closest_point_aabb).length();

    if distance <= capsule.radius {
        let depth = capsule.radius - distance;
        let normal = (closest_point_capsule - closest_point_aabb).normalize();
        let position = closest_point_aabb + normal * depth;

        Some(Collision {
            position: position.into(),
            normal,
            depth,
        })
    } else {
        None
    }
}

fn closest_point_on_aabb(point: Vec3, min: Vec3, max: Vec3) -> Vec3 {
    Vec3::new(
        point.x.clamp(min.x, max.x),
        point.y.clamp(min.y, max.y),
        point.z.clamp(min.z, max.z),
    )
}

fn closest_point_capsule_aabb(
    capsule_start: Vec3,
    capsule_end: Vec3,
    box_min: Vec3,
    box_max: Vec3,
) -> Vec3 {
    let segment_dir = capsule_end - capsule_start;
    let segment_len_sq = segment_dir.length_squared();

    // Handle degenerate capsule (start == end) — it's just a sphere
    if segment_len_sq == 0.0 {
        return closest_point_on_aabb(capsule_start, box_min, box_max);
    }

    // Parameter along the capsule’s central segment [0..1]
    let mut best_t = 0.0_f32;
    let mut best_point = capsule_start;
    let mut best_dist_sq = f32::INFINITY;

    // Check along each axis to find which projection gives the closest approach
    for axis in 0..3 {
        let start = capsule_start[axis];
        let dir = segment_dir[axis];

        // Skip axes where the segment is parallel
        if dir.abs() < 1e-6 {
            continue;
        }

        // Find where the segment would enter or exit the AABB slab on this axis
        let mut candidate_t = 0.0;

        if start < box_min[axis] {
            candidate_t = (box_min[axis] - start) / dir;
        } else if start > box_max[axis] {
            candidate_t = (box_max[axis] - start) / dir;
        } else {
            // Segment starts within the AABB’s extent on this axis — no clamping needed
            continue;
        }

        // Clamp t to [0, 1]
        candidate_t = candidate_t.clamp(0.0, 1.0);

        // Get the corresponding point on the capsule’s segment
        let segment_point = capsule_start + segment_dir * candidate_t;

        // Get the closest point on the AABB to that segment point
        let box_point = closest_point_on_aabb(segment_point, box_min, box_max);

        // Measure squared distance
        let dist_sq = (segment_point - box_point).length_squared();

        // Keep whichever t gives the smallest distance
        if dist_sq < best_dist_sq {
            best_dist_sq = dist_sq;
            best_t = candidate_t;
            best_point = segment_point;
        }
    }

    best_point
}

//------ AABB-Hull collision ------//

fn check_collision_aabb_hull(
    aabb: &Aabb,
    aabb_position: &WorldCoords,
    hull: &Hull,
    hull_position: &WorldCoords,
    invert_normal: bool,
) -> Option<Collision> {
    None
}

//------ Capsule-Hull collision ------//

fn check_collision_capsule_hull(
    capsule: &Capsule,
    capsule_position: &WorldCoords,
    hull: &Hull,
    hull_position: &WorldCoords,
    invert_normal: bool,
) -> Option<Collision> {
    None
}

#[derive(Debug, Clone, Reflect)]
pub enum ColliderType {
    Aabb(Aabb),
    Capsule(Capsule),
    Hull(Hull),
}

#[derive(Debug, Clone, Reflect)]
pub(super) struct Aabb(pub(super) Vec3);
#[derive(Debug, Clone, Reflect)]
pub(super) struct Capsule {
    pub(super) start: Vec3,
    pub(super) end: Vec3,
    pub(super) radius: f32,
}
#[derive(Debug, Clone, Reflect)]
pub(super) struct Hull(pub(super) Vec<Vec3>);

#[derive(Debug, Clone)]
pub struct Collision {
    pub position: WorldCoords,
    pub normal: Vec3,
    pub depth: f32,
}
