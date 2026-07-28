use bevy::math::Vec3;
use bevy::prelude::TypePath;
use physics::{Collider, ColliderKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct ColliderCodec {
    pub format: u8,
    pub collider: ColliderDataCodec,
}
impl ColliderCodec {
    pub const LATEST_FORMAT: u8 = 1;

    pub fn make_collider(&self, pos: Vec3) -> Collider {
        match self.collider {
            ColliderDataCodec::Cuboid { x, y, z } => Collider::cuboid((x, y, z).into(), pos),
            ColliderDataCodec::ConvexHull(ref points) => Collider::convex_hull(points, pos),
            ColliderDataCodec::Capsule(capsule) => capsule.make_collider(pos),
            ColliderDataCodec::Sphere(radius) => {
                Collider::capsule(Vec3::ZERO, Vec3::ZERO, radius, pos)
            }
        }
    }

    pub fn kind(&self) -> ColliderKind {
        match &self.collider {
            ColliderDataCodec::Cuboid { .. } => ColliderKind::Cuboid,
            ColliderDataCodec::ConvexHull(_) => ColliderKind::ConvexHull,
            ColliderDataCodec::Capsule(_) => ColliderKind::Capsule,
            ColliderDataCodec::Sphere { .. } => ColliderKind::Capsule,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub enum ColliderDataCodec {
    Cuboid {
        x: f32,
        y: f32,
        z: f32,
    },
    ConvexHull(Vec<Vec3>),
    Sphere(f32),
    #[serde(untagged)]
    Capsule(CapsuleCodec),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TypePath)]
#[serde(untagged)]
pub enum CapsuleCodec {
    Oriented { start: Vec3, end: Vec3, radius: f32 },
    Vertical { height: f32, radius: f32 },
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
