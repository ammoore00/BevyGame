use crate::AppSystems;
use crate::asset_tracking::LoadResource;
use crate::game::character::animation::CharacterAnimation;
use crate::game::grid::coords::WorldPosition;
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::prelude::*;
use std::fmt::Debug;

mod animation;
pub mod health;
pub mod player;
pub mod stamina;

pub fn plugin(app: &mut App) {
    app.load_resource::<CharacterAssets>();

    app.add_plugins((
        animation::plugin,
        health::plugin,
        player::plugin,
        stamina::plugin,
    ));
    app.add_systems(Update, (update_state,).in_set(AppSystems::Update));
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
        CharacterState::Idle,
        Facing::default(),
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
pub struct CharacterAssets {}

impl FromWorld for CharacterAssets {
    fn from_world(world: &mut World) -> Self {
        let _assets = world.resource::<AssetServer>();
        Self {}
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
pub enum CharacterState {
    Idle,
    Walking,
    Running,
    Attacking { time_left: f32 },
}

impl CharacterState {
    /// If this state is a movement state which can be canceled into other states
    pub fn is_movement(&self) -> bool {
        matches!(
            self,
            CharacterState::Idle | CharacterState::Walking | CharacterState::Running
        )
    }
}

#[derive(EntityEvent, Debug, Clone, Reflect)]
pub struct CharacterStateEvent {
    entity: Entity,
    new_state: CharacterState,
    prev_state: Option<CharacterState>,
    config: CharacterStateEventConfiguration,
}

impl CharacterStateEvent {
    pub fn new(entity: Entity, new_state: CharacterState) -> Self {
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
    mut query: Query<&mut CharacterState, With<Character>>,
) {
    let Ok(mut state) = query.get_mut(event.entity) else {
        return;
    };

    let prev_state = state.clone();

    if let Some(expected_prev_state) = event.prev_state
        && event.config.fail_on_prev_state_mismatch
        && prev_state != expected_prev_state
    {
        // TODO: proper handling
        panic!(
            "Character state mismatch: expected {:?}, got {:?}",
            expected_prev_state, prev_state
        );
    }

    *state = event.new_state;
}

fn update_state(time: Res<Time>, mut query: Query<&mut CharacterState, With<Character>>) {
    query.iter_mut().for_each(|mut state| {
        if let CharacterState::Attacking { ref mut time_left } = *state {
            *time_left -= time.delta_secs();

            if *time_left <= 0.0 {
                *state = CharacterState::Idle;
            }
        }
    })
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum Facing {
    NorthWest = 0,
    West = 1,
    #[default]
    SouthWest = 2,
    South = 3,
    SouthEast = 4,
    East = 5,
    NorthEast = 6,
    North = 7,
}

impl From<usize> for Facing {
    fn from(index: usize) -> Self {
        match index {
            0 => Self::NorthWest,
            1 => Self::West,
            2 => Self::SouthWest,
            3 => Self::South,
            4 => Self::SouthEast,
            5 => Self::East,
            6 => Self::NorthEast,
            7 => Self::North,
            _ => unreachable!(),
        }
    }
}

impl From<Vec2> for Facing {
    fn from(vec: Vec2) -> Self {
        // Calculate angle in radians (-PI to PI)
        // Note: atan2(z, x) where x is "forward" and z is "right"
        let angle = vec.x.atan2(vec.y);

        // Convert to 0-8 range, where each direction occupies 45 degrees (PI/4 radians)
        // Add PI to shift range from [-PI, PI] to [0, 2*PI]
        // Add PI/8 to center the divisions on the cardinal directions
        // Add 3PI/2 to rotate divisions to align with sprite sheets
        // Divide by PI/4 (45 degrees) to get 0-8 range
        let direction_index = ((angle
            + std::f32::consts::PI
            + std::f32::consts::FRAC_PI_8
            + std::f32::consts::FRAC_PI_2 * 3.0)
            / std::f32::consts::FRAC_PI_4)
            .floor() as i32
            % 8;

        Self::from(direction_index as usize)
    }
}
