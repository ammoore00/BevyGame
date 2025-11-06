use std::fmt::Debug;
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
    app.add_observer(on_state_change);
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
        CharacterStateContainer(CharacterState::Idle),
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

#[derive(Component, Asset, Clone, Reflect)]
pub struct CharacterStateContainer(CharacterState);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum CharacterState {
    Idle,
    Walking,
    Running,
}

#[derive(EntityEvent, Debug, Clone, Reflect)]
pub struct CharacterStateEvent {
    entity: Entity,
    new_state: CharacterState,
    prev_state: Option<CharacterState>,
    config: CharacterStateEventConfiguration,
}

impl CharacterStateEvent {
    pub fn new(
        entity: Entity,
        new_state: CharacterState,
    ) -> Self {
        Self {
            entity,
            new_state,
            prev_state: None,
            config: CharacterStateEventConfiguration::default(),
        }
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct CharacterStateEventConfiguration {
    fail_on_prev_state_mismatch: bool,
}

impl Default for CharacterStateEventConfiguration {
    fn default() -> Self {
        Self {
            fail_on_prev_state_mismatch: true,
        }
    }
}

fn on_state_change(
    event: On<CharacterStateEvent>,
    mut query: Query<(&mut CharacterStateContainer), With<Character>>,
) {
    let Ok(mut state) = query.get_mut(event.entity) else {
        return;
    };

    let prev_state = state.0;

    if let Some(expected_prev_state) = event.prev_state && event.config.fail_on_prev_state_mismatch && prev_state != expected_prev_state {
        // TODO: proper handling
        panic!("Character state mismatch: expected {:?}, got {:?}", expected_prev_state, prev_state);
    }

    state.0 = event.new_state;
}