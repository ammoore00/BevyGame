use crate::AppSystems;
use crate::asset_tracking::LoadResource;
use crate::game::character::animation::CharacterAnimation;
use crate::game::grid::coords::WorldPosition;
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::prelude::*;
use std::any::TypeId;
use std::convert::Infallible;
use std::error::Error;
use std::fmt::Debug;
use bevy::ecs::world::DeferredWorld;

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
        character_state(default_states::Idle::default()),
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

pub fn character_state(state: impl CharacterState + Component) -> impl Bundle {
    (
        CharacterStateTracker {
            type_id: state.type_id(),
        },
        state,
    )
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CharacterStateTracker {
    type_id: TypeId,
}

pub trait CharacterState: Reflect + Send + Sync + Debug + 'static {
    fn as_reflect(&self) -> &dyn Reflect;
    fn clone_value(&self) -> Box<dyn Reflect>;
    fn box_clone(&self) -> Box<dyn CharacterState>;
}
pub trait MovementState: CharacterState {}

#[reflect_trait]
pub trait TimedState: CharacterState {
    fn time_left(&self) -> f32;
    fn set_time(&mut self, time: f32);
    fn reset_time(&mut self);
}

pub trait TransitionTo<T: CharacterState> {
    type Err: Error;

    fn can_transition_to(&self, state: &T) -> bool {
        true
    }
    fn on_transition_to(&self, state: &T) -> Result<(), Self::Err> {
        Ok(())
    }
}

/// Marker trait - allows transition from Idle into State
pub trait FromIdle: CharacterState {}
/// Marker trait - allows transition from Movement into State
pub trait FromMovement: CharacterState {}

pub trait InterruptPolicy {
    fn can_interrupt() -> bool;
}
pub struct NoInterrupt;
impl InterruptPolicy for NoInterrupt {
    fn can_interrupt() -> bool {
        false
    }
}
pub struct Interrupt;
impl InterruptPolicy for Interrupt {
    fn can_interrupt() -> bool {
        true
    }
}

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

        impl CharacterState for $name {
            fn as_reflect(&self) -> &dyn Reflect { self }
            fn clone_value(&self) -> Box<dyn Reflect> {
                Box::new(self.clone())
            }
        }

        // Handle all default trait implementations
        define_character_states!(@handle_traits $name $( $(?$traits)* )?);

        define_character_states!($($rest)*);
    };

    // 3. Case: State WITHOUT fields
    ($name:ident $(: $(?$traits:ident)*)? ; $($rest:tt)*) => {
        #[derive(Component, Debug, Clone, Reflect, Default)]
        #[reflect(Component)]
        pub struct $name {}

        impl CharacterState for $name {
            fn as_reflect(&self) -> &dyn Reflect { self }
            fn clone_value(&self) -> Box<dyn Reflect> {
                Box::new(self.clone())
            }
            fn box_clone(&self) -> Box<dyn CharacterState> { Box::new(self.clone()) }
        }

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
    }

    impl<T: FromIdle> TransitionTo<T> for Idle {
        type Err = Infallible;
    }

    impl MovementState for Walking {}
    impl MovementState for Running {}
    impl MovementState for Sprinting {}

    impl<ToState: FromMovement, FromState: MovementState> TransitionTo<ToState> for FromState {
        type Err = Infallible;
    }

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, TimedState)]
    pub struct Attacking { time_left: f32 }

    impl CharacterState for Attacking {
        fn as_reflect(&self) -> &dyn Reflect { self }
        fn clone_value(&self) -> Box<dyn Reflect> {
            Box::new(self.clone())
        }
        fn box_clone(&self) -> Box<dyn CharacterState> { Box::new(self.clone()) }
    }

    impl TimedState for Attacking {
        fn time_left(&self) -> f32 { self.time_left }

        fn set_time(&mut self, time: f32) {
            todo!()
        }

        fn reset_time(&mut self) {
            todo!()
        }
    }

    impl<T: FromAttacking> TransitionTo<T> for Attacking {
        type Err = Infallible;
        fn can_transition_to(&self, _: &T) -> bool {
            <T::Policy as InterruptPolicy>::can_interrupt() || self.time_left <= 0.0
        }
    }
}

#[derive(EntityEvent, Debug)]
pub struct CharacterStateEvent {
    entity: Entity,
    new_state: Box<dyn CharacterState>,
    prev_state: Box<dyn CharacterState>,
}

impl CharacterStateEvent {
    pub fn new(
        entity: Entity,
        new_state: Box<dyn CharacterState>,
        prev_state: Box<dyn CharacterState>,
    ) -> Self {
        Self {
            entity,
            new_state: new_state.into(),
            prev_state: prev_state.into(),
        }
    }
}

fn on_state_change(
    event: On<CharacterStateEvent>,
    mut world: DeferredWorld,
) {
    let entity = event.entity;

    // We clone these to move them into the command closure
    let new_state = event.new_state.clone_value();
    let prev_type_id = event.prev_state.type_id();
    let new_type_id = new_state.type_id();

    // Use the queue to get full World access after the observer logic
    world.commands().queue(move |world: &mut World| {
        let registry = world.resource::<AppTypeRegistry>().clone();
        let type_registry = registry.read();

        if let Some(prev_type) = type_registry.get(prev_type_id)
            && let Some(prev_reflect_component) = prev_type.data::<ReflectComponent>()
            && let Some(next_type) = type_registry.get(new_type_id)
            && let Some(next_reflect_component) = next_type.data::<ReflectComponent>()
            && let Ok(mut entity_mut) = world.get_entity_mut(entity)
        {
            // Remove the old state
            prev_reflect_component.remove(&mut entity_mut);

            // Insert the new state
            next_reflect_component.insert(
                &mut entity_mut,
                new_state.as_reflect(),
                &type_registry,
            );

            // Update the tracker component
            entity_mut.insert(CharacterStateTracker {
                type_id: new_type_id,
            });
        } else {
            warn!("Failed to update state for entity {}: ", entity);
        }
    });
}

fn update_state(
    time: Res<Time>,
    mut commands: Commands,
    registry: Res<AppTypeRegistry>,
    query: Query<(Entity, &CharacterStateTracker), With<Character>>,
) {
    let delta = time.delta_secs();
    let type_registry = registry.read();

    for (entity, tracker) in &query {
        // Find the type registration for the current state
        let Some(registration) = type_registry.get(tracker.type_id) else { continue };

        // ONLY proceed if this type was registered with TimedState reflection
        let Some(_) = registration.data::<ReflectTimedState>() else { continue };
        let Some(_) = registration.data::<ReflectComponent>() else { continue };

        let type_id = tracker.type_id;

        // Perform the update via command queue to get EntityWorldMut
        commands.queue(move |world: &mut World| {
            let type_registry = world.resource::<AppTypeRegistry>().clone();
            let type_registry = type_registry.read();

            // Re-fetch helpers inside closure
            let reg = type_registry.get(type_id).unwrap();
            let reflect_timed_state = reg.data::<ReflectTimedState>().unwrap();
            let reflect_component = reg.data::<ReflectComponent>().unwrap();

            if let Ok(mut entity_mut) = world.get_entity_mut(entity)
                && let Some(reflect_data) = reflect_component.reflect_mut(&mut entity_mut)
                && let Some(timed_state) = reflect_timed_state.get_mut(reflect_data.into_inner())
            {
                let new_time = timed_state.time_left() - delta;
                timed_state.set_time(new_time);

                if new_time > 0.0 {
                    return;
                }

                // To clone, we reach through the Reflected Mut to the underlying data
                let prev_data = timed_state.box_clone();

                world.commands().trigger(CharacterStateEvent::new(
                    entity,
                    Box::new(default_states::Idle::default()),
                    prev_data
                ));
            }
        });
    }
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
