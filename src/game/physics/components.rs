use bevy::prelude::*;

#[derive(Component, Debug, Clone, Reflect)]
pub struct Collider {
    collider_type: ColliderType,
}

impl Collider {
    pub fn aabb(size: Vec3) -> Self {
        Self {
            collider_type: ColliderType::AABB(size),
        }
    }
    
    pub fn capsule(radius: f32, height: f32) -> Self {
        Self {
            collider_type: ColliderType::Capsule {
                radius,
                height,
            },
        }
    }
    
    pub fn hull(points: Vec<Vec3>) -> Self {
        Self {
            collider_type: ColliderType::Hull {
                points,
            },
        }
    }
    
    pub fn get(&self) -> &ColliderType {
        &self.collider_type
    }
}

#[derive(Debug, Clone, Reflect)]
pub enum ColliderType {
    AABB(Vec3),
    Capsule {
        radius: f32,
        height: f32,
    },
    Hull {
        points: Vec<Vec3>,
    }
}