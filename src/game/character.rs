use bevy::prelude::*;
use crate::asset_tracking::LoadResource;
use crate::game::grid::coords::WorldPosition;
use crate::game::physics::components::{Collider, PhysicsData};

pub fn plugin(app: &mut App) {
    app.load_resource::<CharacterAssets>();
}

pub fn sariel(
    assets: &CharacterAssets,
    position: Vec3,
    scale: f32,
) -> impl Bundle {
    (
        Name::new("Sariel"),
        Character,
        WorldPosition(position.into()),
        // Physics
        Collider::vertical_capsule(1.75, 0.375, position),
        PhysicsData::kinematic(Vec3::ZERO),
        // Rendering
        Transform::from_scale(Vec3::splat(scale)),
        Sprite::from(assets.sariel.clone()),
    )
}

#[derive(Component, Asset, Clone, Reflect)]
pub struct Character;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct CharacterAssets {
    #[dependency]
    pub sariel: Handle<Image>,
}

impl FromWorld for CharacterAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            sariel: assets.load("images/sariel.png"),
        }
    }
}