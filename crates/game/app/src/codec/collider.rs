use bevy::math::Vec3;
use bevy::prelude::TypePath;
use serde::{Deserialize, Serialize};
use crate::game::physics::components::Collider;

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct ColliderCodec {
    pub format: u8,
    pub collider: ColliderKindCodec,
}

impl ColliderCodec {
    pub const LATEST_FORMAT: u8 = 1;
}

impl ColliderCodec {
    pub fn make_collider(&self, pos: Vec3) -> Collider {
        match &self.collider {
            ColliderKindCodec::Cuboid { x, y, z } => Collider::cuboid((*x, *y, *z).into(), pos),
            ColliderKindCodec::ConvexHull(points) => Collider::convex_hull(points, pos),
            ColliderKindCodec::Capsule(capsule) => capsule.make_collider(pos),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub enum ColliderKindCodec {
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