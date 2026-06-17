use crate::game::level::grid::coords::{WorldCoords, WorldPosition};
use bevy::prelude::*;
use parry3d::math::Pose;
use parry3d::query;
use parry3d::query::Contact;
use parry3d::shape::{Capsule, ConvexPolyhedron, Cuboid, Shape};
use parry3d::transformation::convex_hull;
use serde::{Deserialize, Serialize};

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
        last_grounded_height: f32,
    },
}

impl PhysicsData {
    pub fn kinematic(displacement: Vec3) -> Self {
        Self::Kinematic {
            displacement,
            grounded: false,
            time_since_grounded: f32::INFINITY,
            last_grounded_height: f32::NAN,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColliderType {
    Cuboid(Cuboid),
    Capsule(Capsule),
    ConvexHull {
        shape: ConvexPolyhedron,
        vertices: Box<Vec<Vec3>>,
        indices: Box<Vec<[u32; 3]>>,
    },
}
impl ColliderType {
    fn get_shape(&self) -> &dyn Shape {
        match &self {
            ColliderType::Cuboid(cuboid) => cuboid,
            ColliderType::Capsule(capsule) => capsule,
            ColliderType::ConvexHull { shape, .. } => shape,
        }
    }
}
impl PartialEq for ColliderType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ColliderType::Cuboid(a), ColliderType::Cuboid(b)) => a == b,
            (ColliderType::Capsule(a), ColliderType::Capsule(b)) => {
                a.radius == b.radius && a.segment == b.segment
            }
            (
                ColliderType::ConvexHull { shape: a_shape, .. },
                ColliderType::ConvexHull { shape: b_shape, .. }
            ) => a_shape == b_shape,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct ColliderCodec {
    pub format: u8,
    pub collider: ColliderTypeCodec,
}
impl ColliderCodec {
    pub const LATEST_FORMAT: u8 = 1;
}
impl ColliderCodec {
    pub fn make_collider(&self, pos: Vec3) -> Collider {
        match &self.collider {
            ColliderTypeCodec::Cuboid { x, y, z } => Collider::cuboid((*x, *y, *z).into(), pos),
            ColliderTypeCodec::ConvexHull(points) => Collider::convex_hull(points, pos),
            ColliderTypeCodec::Capsule(capsule) => capsule.make_collider(pos),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub enum ColliderTypeCodec {
    Cuboid {
        x: f32,
        y: f32,
        z: f32,
    },
    ConvexHull(Vec<Vec3>),
    #[serde(untagged)]
    Capsule(CapsuleCodec),
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
#[serde(untagged)]
pub enum CapsuleCodec {
    Oriented {
        start: Vec3,
        end: Vec3,
        radius: f32,
    },
    Vertical {
        height: f32,
        radius: f32,
    },
}
impl CapsuleCodec {
    pub fn make_collider(&self, pos: Vec3) -> Collider {
        match self {
            CapsuleCodec::Oriented { start, end, radius } => {
                Collider::capsule(*start, *end, *radius, pos)
            }
            CapsuleCodec::Vertical { height, radius } => {
                Collider::vertical_capsule(*height, *radius, pos)
            }
        }
    }
}

fn update_collider_position(query: Query<(&mut Collider, &WorldPosition)>) {
    for (mut collider, world_position) in query {
        collider.position = world_position.clone().into();
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Collider {
    collider_type: ColliderType,
    position: Pose,
}
impl Collider {
    pub fn cuboid(size: Vec3, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        let size: Vec3 = Vec3::new(size.x, size.y, size.z);

        Self {
            collider_type: ColliderType::Cuboid(Cuboid::new(size)),
            position: Pose::translation(position.x, position.y, position.z),
        }
    }

    pub fn capsule(start: Vec3, end: Vec3, radius: f32, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        let start = Vec3::new(start.x, start.y, start.z);
        let end = Vec3::new(end.x, end.y, end.z);

        Self {
            collider_type: ColliderType::Capsule(Capsule::new(start, end, radius)),
            position: Pose::translation(position.x, position.y, position.z),
        }
    }

    pub fn vertical_capsule(height: f32, radius: f32, position: impl Into<WorldCoords>) -> Self {
        let segment_length = height - radius * 2.0;

        let start = Vec3::new(0.0, -radius, 0.0);
        let end = Vec3::new(0.0, segment_length - radius, 0.0);

        Self::capsule(start, end, radius, position)
    }

    pub fn convex_hull(vertices: &[Vec3], position: impl Into<WorldCoords>) -> Self {
        let position = position.into();

        let convex_hull = convex_hull(vertices);
        let convex_polyhedron = ConvexPolyhedron::from_convex_hull(convex_hull.0.as_slice());

        let vertices = convex_hull
            .0
            .iter()
            .map(|point| Vec3::new(point.x, point.y, point.z))
            .collect();

        Self {
            collider_type: ColliderType::ConvexHull {
                shape: convex_polyhedron.expect("Failed to create convex hull"),
                vertices: Box::new(vertices),
                indices: Box::new(convex_hull.1),
            },
            position: Pose::translation(position.x, position.y, position.z),
        }
    }

    pub fn with_collider(collider_type: ColliderType, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        Self {
            collider_type,
            position: Pose::translation(position.x, position.y, position.z),
        }
    }

    pub fn collider_type(&self) -> &ColliderType {
        &self.collider_type
    }

    pub fn check_collision(&self, other: &Self) -> Option<CollisionEvent> {
        query::contact(
            &self.position,
            self.collider_type.get_shape(),
            &other.position,
            other.collider_type.get_shape(),
            0.0,
        )
        .ok()
        .flatten()
        .map(CollisionEvent::from)
    }

    pub fn set_position(&mut self, position: impl Into<WorldCoords>) {
        let position = position.into();
        self.position = Pose::translation(position.x, position.y, position.z);
    }

    /// Get the minimum and maximum world coordinates of the collider.
    /// Returns (min, max)
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let (local_min, local_max) = match &self.collider_type {
            ColliderType::Cuboid(cuboid) => {
                let half_extents = cuboid.half_extents;

                (-half_extents, half_extents)
            }
            ColliderType::Capsule(capsule) => {
                let a = capsule.segment.a;
                let b = capsule.segment.b;
                let r = Vec3::splat(capsule.radius);

                (a.min(b) - r, a.max(b) + r)
            }
            ColliderType::ConvexHull { vertices, .. } => {
                let Some(first_vertex) = vertices.first() else {
                    return (Vec3::ZERO, Vec3::ZERO);
                };

                let mut min = *first_vertex;
                let mut max = *first_vertex;

                for &vertex in vertices.iter().skip(1) {
                    min = min.min(vertex);
                    max = max.max(vertex);
                }

                (min, max)
            }
        };

        let translation = Vec3::new(
            self.position.translation.x,
            self.position.translation.y,
            self.position.translation.z,
        );

        (local_min + translation, local_max + translation)
    }

    /// Get the size of the collider's world-space bounds.
    pub fn size(&self) -> Vec3 {
        let (min, max) = self.bounds();
        max - min
    }
}

#[derive(Debug, Clone)]
pub struct CollisionEvent(Contact);
impl CollisionEvent {
    pub fn _contact_points(&self) -> (Vec3, Vec3) {
        let contact = &self.0;

        let p1 = contact.point1;
        let p2 = contact.point2;

        let p1 = Vec3::new(p1.x, p1.y, p1.z);
        let p2 = Vec3::new(p2.x, p2.y, p2.z);

        (p1, p2)
    }

    pub fn _depth(&self) -> f32 {
        -self.0.dist
    }

    pub fn normal(&self) -> Vec3 {
        Vec3::new(self.0.normal2.x, self.0.normal2.y, self.0.normal2.z)
    }
}
impl From<Contact> for CollisionEvent {
    fn from(contact: Contact) -> Self {
        Self(contact)
    }
}