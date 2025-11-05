use crate::asset_tracking::LoadResource;
use crate::game::character::animation::CharacterAnimation;
use crate::game::grid::coords::WorldPosition;
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::prelude::*;

mod animation;
pub mod health;
pub mod player;

pub fn plugin(app: &mut App) {
    app.load_resource::<CharacterAssets>();

    app.add_plugins((animation::plugin, health::plugin, player::plugin));
}

pub fn character(
    name: impl Into<String>,
    position: Vec3,
    sprite: Sprite,
    animation: CharacterAnimation,
    collider: Collider,
    scale: f32,
) -> impl Bundle {
    (
        Name::new(name.into()),
        Character,
        // Physics
        WorldPosition(position.into()),
        PhysicsData::kinematic(Vec3::ZERO),
        collider,
        // Rendering
        Transform::from_scale(Vec3::splat(scale)),
        sprite,
        animation,
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
