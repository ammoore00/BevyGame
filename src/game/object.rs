use crate::asset_tracking::LoadResource;
use crate::game::grid::coords::{WorldCoords, WorldPosition};
use bevy::prelude::*;

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
    collider: ColliderType,
) -> impl Bundle {
    let shadow = assets.boulder_shadow.clone();

    (
        Object(object_type),
        WorldPosition(position.into()),
        Collider(collider, WorldPosition(position.into())),
        Sprite::from(assets.boulder.clone()),
        Transform::from_scale(Vec3::splat(scale)),
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

#[derive(Debug, Clone, Reflect)]
pub enum ColliderType {
    Rectangle(Vec3),
    Cylinder { radius: f32, height: f32 },
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Collider(pub ColliderType, pub WorldPosition);

impl Collider {
    pub fn is_pos_within_collider(&self, pos: impl Into<WorldCoords>) -> bool {
        false
    }
}
