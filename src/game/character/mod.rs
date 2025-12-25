use crate::AppSystems;
use crate::asset_tracking::LoadResource;
use crate::game::character::animation::CharacterAnimation;
use crate::game::grid::coords::WorldPosition;
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::prelude::*;
use std::any::TypeId;
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
        character_state(default_states::Idle),
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

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct CharacterStateTracker {
    type_id: TypeId,
}

pub trait CharacterStateMarker: Reflect + Send + Sync + Debug + 'static {
    fn set_animation(&self, animation: &mut CharacterAnimation);
}
#[reflect_trait]
pub trait CharacterState: Reflect + Send + Sync + Debug + 'static {
    fn as_reflect(&self) -> &dyn Reflect;
    fn clone_value(&self) -> Box<dyn Reflect>;
    fn box_clone(&self) -> Box<dyn CharacterState>;
    fn set_animation(&self, animation: &mut CharacterAnimation);
}
impl <T: CharacterStateMarker + Clone> CharacterState for T {
    fn as_reflect(&self) -> &dyn Reflect { self }
    fn clone_value(&self) -> Box<dyn Reflect> { Box::new(self.clone()) }
    fn box_clone(&self) -> Box<dyn CharacterState> { Box::new(self.clone()) }
    fn set_animation(&self, animation: &mut CharacterAnimation) {
        CharacterStateMarker::set_animation(self, animation);
    }
}

#[reflect_trait]
pub trait MovementState: CharacterState {}

#[reflect_trait]
pub trait TimedState: CharacterState {
    fn time_left(&self) -> f32;
    fn set_time(&mut self, time: f32);
}

pub trait TransitionTo<T: CharacterState> {
    fn can_transition_to(&self, state: &T) -> bool {
        true
    }
    fn on_transition_to(&self, state: &T) {}
}

#[derive(thiserror::Error, Debug)]
pub enum StateTransitionError {
    #[error("Invalid transition from {from:?} to {to:?}: {reason}")]
    InvalidTransition {
        from: Box<dyn CharacterState>,
        to: Box<dyn CharacterState>,
        reason: String,
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

pub mod default_states {
    use super::*;

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, CharacterState)]
    pub struct Idle;
    impl FromIdle for Idle {}
    impl FromMovement for Idle {}
    impl FromAttacking for Idle { type Policy = NoInterrupt; }
    impl CharacterStateMarker for Idle {
        fn set_animation(&self, animation: &mut CharacterAnimation) {
            animation.set_idle();
        }
    }

    impl<T: FromIdle> TransitionTo<T> for Idle {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, CharacterState, MovementState)]
    pub struct Walking;
    impl FromIdle for Walking {}
    impl FromMovement for Walking {}
    impl FromAttacking for Walking { type Policy = Interrupt; }
    impl CharacterStateMarker for Walking {
        fn set_animation(&self, animation: &mut CharacterAnimation) {
            animation
                .set_walking()
                .unwrap_or_else(|_| animation.set_idle())
        }
    }
    impl MovementState for Walking {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, CharacterState, MovementState)]
    pub struct Running;
    impl FromIdle for Running {}
    impl FromMovement for Running {}
    impl FromAttacking for Running { type Policy = Interrupt; }
    impl CharacterStateMarker for Running {
        fn set_animation(&self, animation: &mut CharacterAnimation) {
            animation
                .set_running()
                .unwrap_or_else(|_| animation.set_idle())
        }
    }
    impl MovementState for Running {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, CharacterState, MovementState)]
    pub struct Sprinting;
    impl FromIdle for Sprinting {}
    impl FromMovement for Sprinting {}
    impl FromAttacking for Sprinting { type Policy = Interrupt; }
    impl CharacterStateMarker for Sprinting {
        fn set_animation(&self, animation: &mut CharacterAnimation) {
            animation
                .set_sprinting()
                .unwrap_or_else(|_| animation.set_idle())
        }
    }
    impl MovementState for Sprinting {}

    impl<From: MovementState, To: FromMovement> TransitionTo<To> for From {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, CharacterState, TimedState)]
    pub struct Attacking { pub time_left: f32 }
    impl FromIdle for Attacking {}
    impl FromMovement for Attacking {}
    impl CharacterStateMarker for Attacking {
        fn set_animation(&self, animation: &mut CharacterAnimation) {
            animation
                .set_attacking()
                .unwrap_or_else(|_| animation.set_idle())
        }
    }
    impl TimedState for Attacking {
        fn time_left(&self) -> f32 { self.time_left }

        fn set_time(&mut self, time: f32) {
            self.time_left = time;
        }
    }

    impl<T: FromAttacking> TransitionTo<T> for Attacking {
        fn can_transition_to(&self, _: &T) -> bool {
            <T::Policy as InterruptPolicy>::can_interrupt() || self.time_left <= 0.0
        }
    }
}

pub fn get_state(entity: Entity, tracker: &CharacterStateTracker, world: &mut World) -> Option<Box<dyn CharacterState>> {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    let reg = type_registry.get(tracker.type_id).unwrap();
    let reflect_component = reg.data::<ReflectComponent>().unwrap();
    let reflect_state = reg.data::<ReflectCharacterState>().unwrap();

    if let Ok(mut entity_mut) = world.get_entity_mut(entity)
        && let Some(reflect_data) = reflect_component.reflect_mut(&mut entity_mut)
        && let Some(state) = reflect_state.get_mut(reflect_data.into_inner())
    {
        Some(state.box_clone())
    } else {
        warn!("Failed to get reflect component for entity {}", entity);
        None
    }
}

#[derive(EntityEvent, Debug)]
pub struct CharacterStateEvent {
    entity: Entity,
    new_state: Box<dyn CharacterState>,
    prev_state: Box<dyn CharacterState>,
}

impl CharacterStateEvent {
    pub fn try_new(
        entity: Entity,
        new_state: Box<dyn CharacterState>,
        prev_state: Box<dyn CharacterState>,
    ) -> Result<Self, StateTransitionError> {
        // TODO: Check state transition logic here
        Ok(Self {
            entity,
            new_state,
            prev_state,
        })
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

                match CharacterStateEvent::try_new(
                    entity,
                    Box::new(default_states::Idle),
                    prev_data
                ) {
                    Ok(event) => {
                        world.commands().trigger(event);
                    },
                    Err(_) => warn!("Failed to transition to Idle state for entity {}", entity)
                }
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
