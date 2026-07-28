use crate::Impulse;
use crate::collision::MAX_COLLISION_DISTANCE;
use crate::math::{ToBevy, ToParry};
use bevy::prelude::*;
use common::{WorldCoords, WorldPosition};
use parry3d::math::Pose;
use parry3d::query;
use parry3d::query::Contact;
use parry3d::shape::{Capsule, ConvexPolyhedron, Cuboid, Shape};
use parry3d::transformation::convex_hull;
use std::collections::VecDeque;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, update_collider_position);
}

/// Keep the collider position up to date with the world position.
fn update_collider_position(query: Query<(&mut Collider, &WorldPosition)>) {
    for (mut collider, pos) in query {
        collider.position = Pose::translation(pos.0.x, pos.0.y, pos.0.z);
    }
}

#[derive(Component, Debug, Clone, PartialEq, Default)]
pub enum PhysicsData {
    /// Physics objects that do not move and do not check for collisions.
    #[default]
    Static,
    /// Moving physics objects that check for collisions against other physics objects.
    Kinematic(KinematicData),
    /// Physics objects that do not cause collisions,
    /// but do trigger events when they touch other physics objects.
    Detector,
}
impl PhysicsData {
    pub fn kinematic() -> Self {
        Self::Kinematic(KinematicData {
            velocity: Vec3::ZERO,
            next_velocity: Vec3::ZERO,

            impulses: VecDeque::new(),

            grounded: false,
            time_since_grounded: f32::INFINITY,
            last_grounded_height: f32::NAN,
            ground_normal: None,
        })
    }

    pub fn kind(&self) -> PhysicsKind {
        match self {
            PhysicsData::Static => PhysicsKind::Static,
            PhysicsData::Kinematic(_) => PhysicsKind::Kinematic,
            PhysicsData::Detector => PhysicsKind::Detector,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicsKind {
    Static,
    Kinematic,
    Detector,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KinematicData {
    /// The current velocity of this object
    pub velocity: Vec3,
    /// The intended next velocity for the entity to move on the current frame.
    /// This will be modified by forces and collision data before being set as the current velocity.
    pub next_velocity: Vec3,

    /// The impulses applied to the entity this frame.
    pub impulses: VecDeque<Impulse>,

    /// Whether the entity is currently touching the ground.
    pub grounded: bool,
    /// The time since the entity last touched the ground.
    pub time_since_grounded: f32,
    /// The height of the entity's feet when it last touched the ground.
    pub last_grounded_height: f32,
    /// The normal of the ground the entity is currently touching, if any.
    pub ground_normal: Option<Vec3>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColliderKind {
    Cuboid,
    Capsule,
    ConvexHull,
}

#[derive(Debug, Clone)]
pub enum ColliderData {
    Cuboid(Cuboid),
    Capsule(Capsule),
    ConvexHull {
        shape: ConvexPolyhedron,
        vertices: Box<Vec<Vec3>>,
        indices: Box<Vec<[u32; 3]>>,
    },
}
impl ColliderData {
    fn get_shape(&self) -> &dyn Shape {
        match &self {
            ColliderData::Cuboid(cuboid) => cuboid,
            ColliderData::Capsule(capsule) => capsule,
            ColliderData::ConvexHull { shape, .. } => shape,
        }
    }
}
impl PartialEq for ColliderData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ColliderData::Cuboid(a), ColliderData::Cuboid(b)) => a == b,
            (ColliderData::Capsule(a), ColliderData::Capsule(b)) => {
                a.radius == b.radius && a.segment == b.segment
            }
            (
                ColliderData::ConvexHull { shape: a_shape, .. },
                ColliderData::ConvexHull { shape: b_shape, .. },
            ) => a_shape == b_shape,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapsuleData {
    pub a: Vec3,
    pub b: Vec3,
    pub radius: f32,
}
impl From<Capsule> for CapsuleData {
    fn from(capsule: Capsule) -> Self {
        let a = Vec3::new(
            capsule.segment.a.x,
            capsule.segment.a.y,
            capsule.segment.a.z,
        );

        let b = Vec3::new(
            capsule.segment.b.x,
            capsule.segment.b.y,
            capsule.segment.b.z,
        );

        let radius = capsule.radius;

        Self { a, b, radius }
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Collider {
    collider_type: ColliderData,
    position: Pose,
}
impl Collider {
    fn new(collider_type: ColliderData, position: Pose) -> Self {
        let slf = Self {
            collider_type,
            position,
        };

        if slf.size().max_element() > MAX_COLLISION_DISTANCE {
            warn!(
                "Collider size exceeds maximum collision distance! This may result in incorrect collision detection.\nMax: {}, Provided: {}",
                MAX_COLLISION_DISTANCE,
                slf.size().max_element()
            )
        }
        slf
    }

    pub fn cuboid(size: Vec3, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        let size: Vec3 = Vec3::new(size.x, size.y, size.z);

        Self::new(
            ColliderData::Cuboid(Cuboid::new(size.to_parry())),
            Pose::translation(position.x, position.y, position.z),
        )
    }

    pub fn capsule(start: Vec3, end: Vec3, radius: f32, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        let start = Vec3::new(start.x, start.y, start.z);
        let end = Vec3::new(end.x, end.y, end.z);

        Self::new(
            ColliderData::Capsule(Capsule::new(start.to_parry(), end.to_parry(), radius)),
            Pose::translation(position.x, position.y, position.z),
        )
    }

    pub fn vertical_capsule(height: f32, radius: f32, position: impl Into<WorldCoords>) -> Self {
        let segment_length = height - radius * 2.0;

        let start = Vec3::new(0.0, -radius, 0.0);
        let end = Vec3::new(0.0, segment_length - radius, 0.0);

        Self::capsule(start, end, radius, position)
    }

    pub fn convex_hull(vertices: &[Vec3], position: impl Into<WorldCoords>) -> Self {
        let position = position.into();

        let vertices = vertices.iter().map(|v| v.to_parry()).collect::<Vec<_>>();

        let convex_hull = convex_hull(vertices.as_slice());
        let convex_polyhedron = ConvexPolyhedron::from_convex_hull(convex_hull.0.as_slice());

        let vertices = convex_hull
            .0
            .iter()
            .map(|point| Vec3::new(point.x, point.y, point.z))
            .collect();

        Self::new(
            ColliderData::ConvexHull {
                shape: convex_polyhedron.expect("Failed to create convex hull"),
                vertices: Box::new(vertices),
                indices: Box::new(convex_hull.1),
            },
            Pose::translation(position.x, position.y, position.z),
        )
    }

    pub fn with_collider(collider_type: ColliderData, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        Self::new(
            collider_type,
            Pose::translation(position.x, position.y, position.z),
        )
    }

    pub fn collider_type(&self) -> &ColliderData {
        &self.collider_type
    }

    pub fn check_collision(&self, other: &Self) -> Option<CollisionContact> {
        query::contact(
            &self.position,
            self.collider_type.get_shape(),
            &other.position,
            other.collider_type.get_shape(),
            0.0,
        )
        .ok()
        .flatten()
        .map(CollisionContact::from)
    }

    pub fn set_position(&mut self, position: impl Into<WorldCoords>) {
        let position = position.into();
        self.position = Pose::translation(position.x, position.y, position.z);
    }

    /// Get the minimum and maximum world coordinates of the collider.
    /// Returns (min, max)
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let (local_min, local_max) = match &self.collider_type {
            ColliderData::Cuboid(cuboid) => {
                let half_extents = cuboid.half_extents.to_bevy();

                (-half_extents, half_extents)
            }
            ColliderData::Capsule(capsule) => {
                let a = capsule.segment.a.to_bevy();
                let b = capsule.segment.b.to_bevy();
                let r = Vec3::splat(capsule.radius);

                (a.min(b) - r, a.max(b) + r)
            }
            ColliderData::ConvexHull { vertices, .. } => {
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
pub struct CollisionContact(Contact);
impl CollisionContact {
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

    pub fn invert_normal(&mut self) {
        self.0.normal2 = -self.0.normal2;
    }

    pub fn with_inverted_normal(&self) -> Self {
        Self(Contact {
            normal2: -self.0.normal2,
            ..self.0
        })
    }
}
impl From<Contact> for CollisionContact {
    fn from(contact: Contact) -> Self {
        Self(contact)
    }
}
