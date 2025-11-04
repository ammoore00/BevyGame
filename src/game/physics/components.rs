use crate::game::grid::coords::{WorldCoords, WorldPosition};
use bevy::prelude::*;
use parry3d::math::Isometry;
use parry3d::na::{Const, OPoint, Vector3};
use parry3d::query;
use parry3d::query::Contact;
use parry3d::shape::{Capsule, ConvexPolyhedron, Cuboid, Shape};
use parry3d::transformation::convex_hull;

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
    ConvexHull(ConvexPolyhedron),
}

impl ColliderType {
    fn get_shape(&self) -> &dyn Shape {
        match &self {
            ColliderType::Cuboid(cuboid) => cuboid,
            ColliderType::Capsule(capsule) => capsule,
            ColliderType::ConvexHull(convex_hull) => convex_hull,
        }
    }
}

fn update_collider_position(query: Query<(&mut Collider, &WorldPosition)>) {
    for (mut collider, world_position) in query {
        collider.position = world_position.clone().into();
    }
}

#[derive(Component, Debug, Clone)]
pub struct Collider {
    collider_type: ColliderType,
    position: Isometry<f32>,
}

impl Collider {
    pub fn cuboid(size: Vec3, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        let size: Vector3<f32> = Vector3::new(size.x, size.y, size.z);

        Self {
            collider_type: ColliderType::Cuboid(Cuboid::new(size)),
            position: Isometry::translation(position.x, position.y, position.z),
        }
    }

    pub fn capsule(start: Vec3, end: Vec3, radius: f32, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        let start = OPoint::<f32, Const<3>>::new(start.x, start.y, start.z);
        let end = OPoint::<f32, Const<3>>::new(end.x, end.y, end.z);

        Self {
            collider_type: ColliderType::Capsule(Capsule::new(start, end, radius)),
            position: Isometry::translation(position.x, position.y, position.z),
        }
    }

    pub fn vertical_capsule(height: f32, radius: f32, position: impl Into<WorldCoords>) -> Self {
        let segment_length = height - radius * 2.0;

        let start = Vec3::new(0.0, -radius, 0.0);
        let end = Vec3::new(0.0, segment_length - radius, 0.0);

        Self::capsule(start, end, radius, position)
    }

    pub fn convex_hull(vertices: Vec<Vec3>, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();

        let vertices = vertices
            .iter()
            .map(|v| OPoint::<f32, Const<3>>::new(v.x, v.y, v.z))
            .collect::<Vec<_>>();

        let convex_hull = convex_hull(vertices.as_slice());
        let convex_polyhedron = ConvexPolyhedron::from_convex_hull(convex_hull.0.as_slice());

        Self {
            collider_type: ColliderType::ConvexHull(
                convex_polyhedron.expect("Failed to create convex hull"),
            ),
            position: Isometry::translation(position.x, position.y, position.z),
        }
    }

    pub fn with_collider(collider_type: ColliderType, position: impl Into<WorldCoords>) -> Self {
        let position = position.into();
        Self {
            collider_type,
            position: Isometry::translation(position.x, position.y, position.z),
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
}

#[derive(Debug, Clone)]
pub struct CollisionEvent(Contact);

impl CollisionEvent {
    pub fn _contact_points(&self) -> (Vec3, Vec3) {
        let contact = &self.0;

        let p1 = contact.point1.coords;
        let p2 = contact.point2.coords;

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
