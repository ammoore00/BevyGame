use std::convert::Infallible;
use std::error::Error;
use crate::AppSystems;
use crate::asset_tracking::LoadResource;
use crate::game::character::animation::CharacterAnimation;
use crate::game::grid::coords::WorldPosition;
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::prelude::*;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

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
        CharacterStateOld::Idle,
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

#[derive(Component, Asset, Clone, Copy, Reflect)]
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

pub trait CharacterState {}
pub trait MovementState: CharacterState {}

pub trait TransitionTo<T: CharacterState> {
    type Err: Error;

    fn can_transition_to(&self, state: &T) -> bool { true }
    fn on_transition_to(&self, state: &T) -> Result<(), Self::Err> { Ok(()) }
}

/// Marker trait - allows transition from Idle into State
pub trait FromIdle: CharacterState {}
/// Marker trait - allows transition from Movement into State
pub trait FromMovement: CharacterState {}

pub trait InterruptPolicy {
    fn can_interrupt() -> bool;
}
pub struct NoInterrupt;
impl InterruptPolicy for NoInterrupt { fn can_interrupt() -> bool { false } }
pub struct Interrupt;
impl InterruptPolicy for Interrupt { fn can_interrupt() -> bool { true } }

/// Marker trait - allows transition from Attacking into State
pub trait FromAttacking: CharacterState {
    type Policy: InterruptPolicy;
}

/// Usage:
/// This macro generates character state types with automatic `CharacterState` trait implementation.
///
/// Automatic traits include:
/// ```ignore
/// FromIdle,
/// FromMovement,
/// FromAttacking
/// ```
/// These automatic traits control blanket implementations of `TransitionTo` for all states.
/// They can be opted out by prefixing them with `?`.
///
/// # Syntax
/// ```ignore
/// define_character_states! {
///     StateName;                                  // State without fields
///     StateName { field: Type };                  // State with fields
///     StateName : ?TraitName;                     // State that does NOT implement TraitName
///     StateName { field: Type } : ?TraitName;     // State with fields that does NOT implement TraitName
///     StateName : ?Trait1 ?Trait2;                // State that does NOT implement multiple traits
/// }
/// ```
///
/// # Examples
/// ```ignore
/// define_character_states! {
///     Idle;                           // Simple state, implements all default traits
///     Walking;                        // Simple state, implements all default traits
///     Attacking { time_left: f32 };   // State with data, implements all default traits
///     Dead : ?FromIdle ?FromMovement; // State that cannot be transitioned to from Idle or Movement
/// }
/// ```
///
/// # Generated Code
/// For each state, this macro generates:
/// - A component struct (with or without fields)
/// - `Component` derive with reflection support
/// - `CharacterState` trait implementation
/// - Default trait implementations (e.g., `FromIdle`, `FromMovement`, `FromAttacking`)
///   - Any trait prefixed with `?` in the opt-out list will NOT be implemented
///   - To add support for new transition traits, add corresponding `@check_*` and `@handle_traits` entries
///
macro_rules! define_character_states {
    // 1. Terminal case
    () => {};

    // 2. Case: State WITH fields and optional trait opt-outs
    ($name:ident { $($field:ident: $type:ty),* $(,)? } $(: $(?$traits:ident)*)? ; $($rest:tt)*) => {
        #[derive(Component, Debug, Clone, Reflect, Default)]
        #[reflect(Component)]
        pub struct $name { $(pub $field: $type),* }
    
        impl CharacterState for $name {}
        
        // Handle all default trait implementations
        define_character_states!(@handle_traits $name $( $(?$traits)* )?);
    
        define_character_states!($($rest)*);
    };

    // 3. Case: State WITHOUT fields
    ($name:ident $(: $(?$traits:ident)*)? ; $($rest:tt)*) => {
        #[derive(Component, Debug, Clone, Reflect, Default)]
        #[reflect(Component)]
        pub struct $name {}
    
        impl CharacterState for $name {}
        
        define_character_states!(@handle_traits $name $( $(?$traits)* )?);
    
        define_character_states!($($rest)*);
    };

    // --- Internal Trait Handlers ---

    (@handle_traits $name:ident $(?$traits:ident)*) => {
        define_character_states!(@check_from_idle $name $(?$traits)*);
        define_character_states!(@check_from_movement $name $(?$traits)*);
        define_character_states!(@check_from_attacking $name $(?$traits)*);
    };

    // FromIdle logic
    (@check_from_idle $name:ident FromIdle $($others:tt)*) => {}; // Found it, skip
    (@check_from_idle $name:ident $ignore:ident $($others:tt)*) => { define_character_states!(@check_from_idle $name $($others)*); };
    (@check_from_idle $name:ident) => { impl FromIdle for $name {} };

    // FromMovement logic
    (@check_from_movement $name:ident FromMovement $($others:tt)*) => {}; // Found it, skip
    (@check_from_movement $name:ident $ignore:ident $($others:tt)*) => { define_character_states!(@check_from_movement $name $($others)*); };
    (@check_from_movement $name:ident) => { impl FromMovement for $name {} };

    // FromAttacking logic
    (@check_from_attacking $name:ident FromAttacking $($others:tt)*) => {}; // Found it, skip
    (@check_from_attacking $name:ident $ignore:ident $($others:tt)*) => { define_character_states!(@check_from_attacking $name $($others)*); };
    (@check_from_attacking $name:ident) => { impl FromAttacking for $name { type Policy = crate::game::character::NoInterrupt; } };
}

pub mod default_states {
    use super::*;

    define_character_states! {
        Idle;
        Walking;
        Running;
        Sprinting;
        Attacking { time_left: f32 };
    }

    impl <T: FromIdle> TransitionTo<T> for Idle {
        type Err = Infallible;
    }

    impl MovementState for Walking {}
    impl MovementState for Running {}
    impl MovementState for Sprinting {}

    impl <ToState: FromMovement, FromState: MovementState> TransitionTo<ToState> for FromState {
        type Err = Infallible;
    }

    impl<T: FromAttacking> TransitionTo<T> for Attacking {
        type Err = Infallible;
        fn can_transition_to(&self, _: &T) -> bool {
            <T::Policy as InterruptPolicy>::can_interrupt() || self.time_left <= 0.0
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
pub enum CharacterStateOld {
    Idle,
    Walking,
    Running,
    Sprinting,
    Attacking { time_left: f32 },
}

impl CharacterStateOld {
    /// If this state is a movement state which can be canceled into other states
    pub fn is_movement(&self) -> bool {
        matches!(
            self,
            CharacterStateOld::Idle
                | CharacterStateOld::Walking
                | CharacterStateOld::Running
                | CharacterStateOld::Sprinting
        )
    }
}

#[derive(EntityEvent, Debug, Clone, Reflect)]
pub struct CharacterStateEvent {
    entity: Entity,
    new_state: CharacterStateOld,
    prev_state: Option<CharacterStateOld>,
    config: CharacterStateEventConfiguration,
}

impl CharacterStateEvent {
    pub fn new(entity: Entity, new_state: CharacterStateOld) -> Self {
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
    mut query: Query<&mut CharacterStateOld, With<Character>>,
) {
    let Ok(mut state) = query.get_mut(event.entity) else {
        return;
    };

    let prev_state = *state;

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

fn update_state(time: Res<Time>, mut query: Query<&mut CharacterStateOld, With<Character>>) {
    query.iter_mut().for_each(|mut state| {
        if let CharacterStateOld::Attacking { ref mut time_left } = *state {
            *time_left -= time.delta_secs();

            if *time_left <= 0.0 {
                *state = CharacterStateOld::Idle;
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
