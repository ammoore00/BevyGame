use crate::asset_tracking::LoadResource;
use crate::game::grid::coords::WorldPosition;
use bevy::prelude::*;
use crate::game::physics::components::{Collider, PhysicsData};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<ObjectAssets>();
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Object(ObjectType);

#[derive(Debug, Clone, Copy, Reflect)]
pub enum ObjectType {
    Rock,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ObjectAssets {
    #[dependency]
    boulder: Handle<Image>,
    #[dependency]
    boulder_shadow: Handle<Image>,
}

impl FromWorld for ObjectAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        ObjectAssets {
            boulder: assets.load("images/boulder.png"),
            boulder_shadow: assets.load("images/boulder_shadow.png"),
        }
    }
}

pub fn object(
    object_type: ObjectType,
    assets: &ObjectAssets,
    position: Vec3,
    scale: f32,
    collider_radius: f32,
    collider_height: f32,
) -> impl Bundle {
    let shadow = assets.boulder_shadow.clone();

    (
        Object(object_type),
        WorldPosition(position.into()),
        Transform::from_scale(Vec3::splat(scale)),
        // Physics
        Collider::capsule(collider_radius, collider_height, position.into()),
        PhysicsData::Static,
        // Rendering
        Sprite::from(assets.boulder.clone()),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((
                Sprite {
                    image: shadow,
                    color: Color::srgba(1.0, 1.0, 1.0, 0.75),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.375 * scale, -0.125 * scale, -0.1)),
            ));
        })),
    )
}
