use crate::PausableSystems;
use crate::game::grid::coords::{WorldCoords, WorldPosition};
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
    pub fn kinematic(displacement: Vec3) -> Self {
        Self::Kinematic {
            displacement,
            grounded: false,
            time_since_grounded: f32::INFINITY,
        }
    }
}

#[derive(Debug, Clone, Reflect)]
pub enum ColliderType {
    Aabb(Aabb),
    Capsule(Capsule),
    Hull(Hull),
}

#[derive(Debug, Clone, Reflect)]
pub struct Aabb(pub(super) Vec3);
#[derive(Debug, Clone, Reflect)]
pub struct Capsule {
    pub(super) start: Vec3,
    pub(super) end: Vec3,
    pub(super) radius: f32,
}

#[derive(Debug, Clone, Reflect)]
pub struct Hull {
    vertices: Vec<Vec3>,
    faces: Vec<Face>,
}

#[derive(Debug, Clone, Reflect)]
struct Face {
    vertices: [usize; 3],
    normal: Vec3,
}

impl Hull {
    pub fn new(vertices: [Vec3; 8]) -> Self {
        Self {
            vertices: Vec::from(vertices),
            faces: Self::get_faces(vertices),
        }
    }

    fn get_faces(vertices: [Vec3; 8]) -> Vec<Face> {
        let pos_y = [4, 5, 6, 7];
        let neg_y = [0, 1, 2, 3];

        let pos_x = [1, 2, 6, 7];
        let neg_x = [0, 4, 5, 3];

        let pos_z = [2, 3, 5, 6];
        let neg_z = [0, 1, 7, 4];

        [pos_y, neg_y, pos_x, neg_x, pos_z, neg_z]
            .iter()
            .flat_map(|quad| Self::split_face(vertices, *quad))
            .collect()
    }

    /// Splits each quad into two triangles
    /// If there are any degeneracies, it may return fewer than two faces
    /// - One face will be returned if a single pair of points overlap
    /// - No faces will be returned if all points are collinear
    fn split_face(vertices: [Vec3; 8], quad: [usize; 4]) -> Vec<Face> {
        let indices = Vec::from(quad);

        // TODO: proper deduplication - right now the indexes will be different even if the points are the same
        let mut deduped_indices = Vec::new();
        let mut deduped_points = Vec::new();

        for point in indices {
            if !deduped_points.contains(&vertices[point]) {
                deduped_indices.push(point);
                deduped_points.push(vertices[point]);
            }
        }

        match deduped_points.len() {
            // All four points are unique
            4 => {
                let face_center =
                    (deduped_points[0] + deduped_points[1] + deduped_points[2] + deduped_points[3])
                        / 4.0;

                let test_normal =
                    compute_normal(&deduped_points[0], &deduped_points[1], &deduped_points[3]);

                // Ensure that the faces are all convex when splitting non-planar faces
                if test_normal.dot(face_center) <= 0.0 {
                    vec![
                        Face {
                            vertices: [0, 1, 3],
                            normal: compute_normal(
                                &deduped_points[0],
                                &deduped_points[1],
                                &deduped_points[3],
                            ),
                        },
                        Face {
                            vertices: [0, 3, 2],
                            normal: compute_normal(
                                &deduped_points[0],
                                &deduped_points[3],
                                &deduped_points[2],
                            ),
                        },
                    ]
                } else {
                    vec![
                        Face {
                            vertices: [0, 1, 3],
                            normal: compute_normal(
                                &deduped_points[0],
                                &deduped_points[2],
                                &deduped_points[3],
                            ),
                        },
                        Face {
                            vertices: [0, 3, 2],
                            normal: compute_normal(
                                &deduped_points[0],
                                &deduped_points[3],
                                &deduped_points[1],
                            ),
                        },
                    ]
                }
            }
            // A single pair of points overlap, so just return one triangle
            3 => {
                let normal =
                    compute_normal(&deduped_points[0], &deduped_points[1], &deduped_points[2]);
                vec![Face {
                    vertices: [deduped_indices[0], deduped_indices[1], deduped_indices[2]],
                    normal,
                }]
            }
            // Less than 2 unique points, so return nothing
            _ => Vec::new(),
        }
    }
}

fn compute_normal(v0: &Vec3, v1: &Vec3, v2: &Vec3) -> Vec3 {
    (*v1 - *v0).cross(*v2 - *v0).normalize()
}

#[derive(Debug, Clone)]
pub struct Collision {
    pub position: WorldCoords,
    pub normal: Vec3,
    pub depth: f32,
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

    /// Provide points in counterclockwise order
    /// Lower points first, then upper
    ///
    /// Counterclockwise defined as looking at that face
    pub fn hull(vertices: [Vec3; 8], position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        Self {
            collider_type: ColliderType::Hull(Hull::new(vertices)),
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

            // AABB-Capsule
            (C::Aabb(aabb), C::Capsule(capsule)) => {
                check_collision_aabb_capsule(aabb, &self.position, capsule, &other.position)
            }
            (C::Capsule(capsule), C::Aabb(aabb)) => {
                check_collision_capsule_aabb(capsule, &self.position, aabb, &other.position)
            }

            // AABB-Hull
            // TODO
            (C::Aabb(_), C::Hull(_)) => None,

            // Capsule-Hull
            (C::Capsule(capsule), C::Hull(hull)) => {
                check_collision_capsule_hull(capsule, &self.position, hull, &other.position)
            }

            // Only tiles use hull collision, and they will never be the source of a collision
            (C::Hull(_), _) => unreachable!("Attempted collision check with hull as source"),
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
            ColliderType::Aabb(aabb) => (aabb.0 / 2.0).length(),
            ColliderType::Capsule(capsule) => {
                (capsule.start - capsule.end).length() / 2.0 + capsule.radius
            }
            ColliderType::Hull(hull) => {
                let mut points: Vec<Vec3> = Vec::new();

                for face in &hull.faces {
                    points.extend(face.vertices.iter().map(|&i| hull.vertices[i]));
                }

                points.dedup();

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
    let start = capsule.start + position.0;
    let end = capsule.end + position.0;
    let other_start = other_capsule.start + other_position.0;
    let other_end = other_capsule.end + other_position.0;

    let (closest_point, closest_point_on_other) =
        closest_points_between_segments(start, end, other_start, other_end);
    let offset = closest_point - closest_point_on_other;
    let distance = offset.length();

    let combined_radius = capsule.radius + other_capsule.radius;

    if distance < combined_radius {
        let depth = combined_radius - distance;

        // Collision normal points from capsule1 -> capsule2
        let normal = if distance > 1e-6 {
            offset.normalize()
        } else {
            // arbitrary fallback if perfectly overlapping
            Vec3::Y
        };

        Some(Collision {
            position: ((closest_point + closest_point_on_other) * 0.5).into(),
            normal,
            depth,
        })
    } else {
        None
    }
}

fn closest_points_between_segments(
    start: Vec3,
    end: Vec3,
    other_start: Vec3,
    other_end: Vec3,
) -> (Vec3, Vec3) {
    // Direction vectors of both segments
    let segment = end - start;
    let other_segment = other_end - other_start;

    // Vector between the two starting points
    let start_diff = start - other_start;

    // Squared lengths of each segment
    let length_sq = segment.dot(segment);
    let other_length_sq = other_segment.dot(other_segment);

    // Handle degenerate cases where one or both segments collapse into points
    const EPS: f32 = 1e-6;
    if length_sq <= EPS && other_length_sq <= EPS {
        return (start, other_start);
    }

    let (mut segment_t, other_segment_t);

    if length_sq <= EPS {
        // Segment 1 is just a point
        segment_t = 0.0;
        other_segment_t = (other_segment.dot(start_diff) / other_length_sq).clamp(0.0, 1.0);
    } else {
        let segment_to_other = segment.dot(start_diff);

        if other_length_sq <= EPS {
            // Segment 2 is just a point
            other_segment_t = 0.0;
            segment_t = (-segment_to_other / length_sq).clamp(0.0, 1.0);
        } else {
            // Both are valid segments
            let seg_dot = segment.dot(other_segment);
            let denom = length_sq * other_length_sq - seg_dot * seg_dot;

            if denom.abs() > EPS {
                // Compute the projected parameter along segment 1
                segment_t = (seg_dot * other_segment.dot(start_diff)
                    - segment_to_other * other_length_sq)
                    / denom;
                segment_t = segment_t.clamp(0.0, 1.0);
            } else {
                // Segments are nearly parallel
                segment_t = 0.0;
            }

            // Compute where segment 2 lies relative to segment 1’s point
            let proj_on_seg2 = seg_dot * segment_t + other_segment.dot(start_diff);

            if proj_on_seg2 < 0.0 {
                // Closest point lies before seg2_start
                other_segment_t = 0.0;
                segment_t = (-segment_to_other / length_sq).clamp(0.0, 1.0);
            } else if proj_on_seg2 > other_length_sq {
                // Closest point lies after seg2_end
                other_segment_t = 1.0;
                segment_t = (seg_dot - segment_to_other) / length_sq;
                segment_t = segment_t.clamp(0.0, 1.0);
            } else {
                other_segment_t = proj_on_seg2 / other_length_sq;
            }
        }
    }

    // Calculate actual closest points
    let closest_point_seg1 = start + segment * segment_t;
    let closest_point_seg2 = other_start + other_segment * other_segment_t;

    (closest_point_seg1, closest_point_seg2)
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
    let segment_len = segment_dir.length();

    // Handle degenerate capsule (start == end) — it's just a sphere
    if segment_len == 0.0 {
        return closest_point_on_aabb(capsule_start, box_min, box_max);
    }

    let mut nearest_point = capsule_start;
    let mut smallest_distance = f32::INFINITY;

    // Check along each axis to find which projection gives the closest approach
    for axis in 0..3 {
        let start = capsule_start[axis];
        let dir = segment_dir[axis];

        // Skip axes where the segment is parallel
        if dir.abs() < 1e-6 {
            continue;
        }

        // Find where the segment would enter or exit the AABB slab on this axis
        let candidate_t = if start < box_min[axis] {
            (box_min[axis] - start) / dir
        } else if start > box_max[axis] {
            (box_max[axis] - start) / dir
        } else {
            // Segment starts within the AABB’s extent on this axis — no clamping needed
            continue;
        };

        // Get the corresponding point on the capsule’s segment
        let segment_point = capsule_start + segment_dir * candidate_t.clamp(0.0, 1.0);

        // Get the closest point on the AABB to that segment point
        let box_point = closest_point_on_aabb(segment_point, box_min, box_max);

        // Keep whichever t gives the smallest distance
        let distance = (segment_point - box_point).length();
        if distance < smallest_distance {
            smallest_distance = distance;
            nearest_point = segment_point;
        }
    }

    nearest_point
}

//------ AABB-Hull collision ------//

fn check_collision_aabb_hull(
    aabb: &Aabb,
    aabb_position: &WorldCoords,
    other_hull: &Hull,
    other_hull_position: &WorldCoords,
) -> Option<Collision> {
    None
}

//------ Capsule-Hull collision ------//

fn check_collision_capsule_hull(
    capsule: &Capsule,
    capsule_position: &WorldCoords,
    other_hull: &Hull,
    other_hull_position: &WorldCoords,
) -> Option<Collision> {
    None
}
