use crate::AppSystems;
use crate::asset_tracking::LoadResource;
use crate::game::character::animation::{AnimationStateMap, CharacterAnimationTracker};
use crate::game::character::state_transitions::{StateCapabilities, StateTransitionError};
use crate::game::level::grid::coords::WorldPosition;
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::{Arc, RwLock};
use tracing::warn;

pub mod animation;
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
    app.add_systems(Update, (update_timed_state,).in_set(AppSystems::Update));
    app.add_observer(on_state_change);

    default_states::register_states(app);
}

pub fn character(
    name: impl Into<String>,
    position: Vec3,
    state_capabilities: StateCapabilities,
    sprite: Sprite,
    animation_tracker: CharacterAnimationTracker,
    animation_map: AnimationStateMap,
    collider: Collider,
    scale: f32,
) -> impl Bundle {
    (
        Name::new(name.into()),
        Character,
        character_state(default_states::Idle),
        state_capabilities,
        Facing::default(),
        // Physics
        WorldPosition(position.into()),
        PhysicsData::kinematic(Vec3::ZERO),
        collider,
        // Rendering
        Transform::from_scale(Vec3::splat(scale)),
        sprite,
        animation_tracker,
        animation_map,
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

pub trait CharacterStateMarker: Reflect + Send + Sync + Debug + 'static {}
#[reflect_trait]
pub trait CharacterState: Reflect + Send + Sync + Debug + 'static {
    fn clone_value(&self) -> Box<dyn Reflect>;
    fn box_clone(&self) -> Box<dyn CharacterState>;
    fn as_any(&self) -> &dyn Any;
}
impl<T: CharacterStateMarker + Clone> CharacterState for T {
    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }
    fn box_clone(&self) -> Box<dyn CharacterState> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[reflect_trait]
pub trait MovementState: CharacterState {}

#[reflect_trait]
pub trait TimedState: CharacterState {
    fn time_left(&self) -> f32;
    fn set_time(&mut self, time: f32);
}

mod state_transitions {
    use super::*;
    use std::collections::HashMap;

    #[derive(Component, Debug, Clone)]
    pub struct StateCapabilities {
        transition_graph: HashMap<(TypeId, TypeId), StateTransitionRule>,
    }

    impl StateCapabilities {
        pub fn new(
            allowed_states: Vec<TypeId>,
            transition_rules: Vec<StateTransitionRule>,
        ) -> Self {
            let mut transition_graph = HashMap::new();

            // Populate transition graph
            for from_state in &allowed_states {
                for to_state in &allowed_states {
                    // Can't transition to self
                    if from_state == to_state {
                        continue;
                    }

                    // Add supplied rules
                    if let Some(rule) = transition_rules
                        .iter()
                        .find(|rule| rule.matches_type(*from_state, *to_state))
                    {
                        transition_graph.insert((*from_state, *to_state), rule.clone());
                    } else {
                        // Finish graph by explicitly disallowing all other transitions
                        transition_graph.insert(
                            (*from_state, *to_state),
                            StateTransitionRule::never(
                                StateMatcher::Single(*from_state),
                                StateMatcher::Single(*to_state),
                            ),
                        );
                    }
                }
            }

            Self { transition_graph }
        }

        pub(super) fn can_transition(
            &self,
            prev: &dyn CharacterState,
            next: &dyn CharacterState,
        ) -> Result<(), StateTransitionError> {
            if let Some(matcher) = self.transition_graph.get(&(prev.type_id(), next.type_id())) {
                matcher.can_transition(prev, next)
            } else {
                Err(StateTransitionError::InvalidTransition {
                    from: prev.box_clone(),
                    to: next.box_clone(),
                    reason: InvalidTransitionReason::StateNotAllowed,
                })
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct StateTransitionRule {
        prev_matcher: StateMatcher,
        next_matcher: StateMatcher,

        transition_checker: StateTransitionChecker,
    }

    impl StateTransitionRule {
        pub fn new(
            prev_matcher: StateMatcher,
            next_matcher: StateMatcher,
            transition_checker: StateTransitionChecker,
        ) -> Self {
            Self {
                prev_matcher,
                next_matcher,
                transition_checker,
            }
        }

        pub fn always(prev_matcher: StateMatcher, next_matcher: StateMatcher) -> Self {
            Self {
                prev_matcher,
                next_matcher,
                transition_checker: StateTransitionChecker::Always,
            }
        }

        fn never(prev_matcher: StateMatcher, next_matcher: StateMatcher) -> Self {
            Self {
                prev_matcher,
                next_matcher,
                transition_checker: StateTransitionChecker::Never,
            }
        }

        fn matches(&self, prev: &dyn CharacterState, next: &dyn CharacterState) -> bool {
            self.prev_matcher.matches(prev) && self.next_matcher.matches(next)
        }

        fn matches_type(&self, prev: TypeId, next: TypeId) -> bool {
            self.prev_matcher.matches_type(prev) && self.next_matcher.matches_type(next)
        }

        fn can_transition(
            &self,
            prev: &dyn CharacterState,
            next: &dyn CharacterState,
        ) -> Result<(), StateTransitionError> {
            if prev.type_id() == next.type_id() {
                return Err(StateTransitionError::SelfTransition);
            }

            if !self.matches(prev, next) {
                return Err(StateTransitionError::InvalidMatcher(format!(
                    "Transition rule does not apply to prev: {:?} next: {:?}",
                    prev, next
                )));
            }

            match &self.transition_checker {
                StateTransitionChecker::Always => Ok(()),
                StateTransitionChecker::Custom(checker) => {
                    checker.read().unwrap()(prev, next).then_some(()).ok_or(
                        StateTransitionError::InvalidTransition {
                            from: prev.box_clone(),
                            to: next.box_clone(),
                            reason: InvalidTransitionReason::BlockedByState(
                                "Blocked by internal state conditions".to_string(),
                            ), // TODO: Better explanation of errors
                        },
                    )
                }
                StateTransitionChecker::Never => Err(StateTransitionError::InvalidTransition {
                    from: prev.box_clone(),
                    to: next.box_clone(),
                    reason: InvalidTransitionReason::IllegalTransition,
                }),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum StateMatcher {
        Single(TypeId),
        Multiple(Vec<TypeId>),
    }

    impl StateMatcher {
        fn matches(&self, state: &dyn CharacterState) -> bool {
            match self {
                Self::Single(matcher) => matcher == &state.type_id(),
                Self::Multiple(matchers) => matchers.contains(&state.type_id()),
            }
        }

        fn matches_type(&self, state_type: TypeId) -> bool {
            match self {
                Self::Single(matcher) => matcher == &state_type,
                Self::Multiple(matchers) => matchers.contains(&state_type),
            }
        }
    }

    #[derive(Clone)]
    pub enum StateTransitionChecker {
        Always,
        Custom(
            Arc<
                RwLock<
                    dyn for<'a> Fn(&'a dyn CharacterState, &'a dyn CharacterState) -> bool
                        + Send
                        + Sync,
                >,
            >,
        ),
        Never,
    }

    impl StateTransitionChecker {
        pub fn custom(
            function: Box<
                dyn for<'a> Fn(&'a dyn CharacterState, &'a dyn CharacterState) -> bool
                    + Send
                    + Sync,
            >,
        ) -> Self {
            Self::Custom(Arc::new(RwLock::new(function)))
        }
    }

    impl Debug for StateTransitionChecker {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Always => write!(f, "Always"),
                Self::Custom(_) => write!(f, "Custom"),
                Self::Never => write!(f, "Never"),
            }
        }
    }

    #[derive(thiserror::Error, Debug)]
    pub enum StateTransitionError {
        #[error("Invalid transition from {from:?} to {to:?}: {reason}")]
        InvalidTransition {
            from: Box<dyn CharacterState>,
            to: Box<dyn CharacterState>,
            reason: InvalidTransitionReason,
        },
        #[error("Invalid matcher: {0}")]
        InvalidMatcher(String),
        #[error("Transition to self not allowed")]
        SelfTransition,
    }

    #[derive(Debug)]
    pub enum InvalidTransitionReason {
        IllegalTransition,
        BlockedByState(String),
        StateNotAllowed,
        _Other(String),
    }

    impl Display for InvalidTransitionReason {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                InvalidTransitionReason::IllegalTransition => write!(f, "Illegal transition"),
                InvalidTransitionReason::BlockedByState(msg) => {
                    write!(f, "Transition blocked: {}", msg)
                }
                InvalidTransitionReason::StateNotAllowed => write!(f, "State not allowed"),
                InvalidTransitionReason::_Other(msg) => write!(f, "Other: {}", msg),
            }
        }
    }
}

pub mod default_states {
    use super::*;
    use crate::game::character::state_transitions::{
        StateMatcher, StateTransitionChecker, StateTransitionRule,
    };
    use std::cell::LazyCell;

    pub(super) fn register_states(app: &mut App) {
        app.register_type::<Idle>();
        app.register_type::<Walking>();
        app.register_type::<Running>();
        app.register_type::<Sprinting>();
        app.register_type::<Attacking>();
    }

    pub const _DEFAULT_STATES: LazyCell<Vec<TypeId>> = LazyCell::new(|| {
        vec![
            TypeId::of::<Idle>(),
            TypeId::of::<Walking>(),
            TypeId::of::<Running>(),
            TypeId::of::<Sprinting>(),
            TypeId::of::<Attacking>(),
        ]
    });

    pub const _DEFAULT_STATES_PASSIVE: LazyCell<Vec<TypeId>> = LazyCell::new(|| {
        vec![
            TypeId::of::<Idle>(),
            TypeId::of::<Walking>(),
            TypeId::of::<Running>(),
            TypeId::of::<Sprinting>(),
        ]
    });

    pub const DEFAULT_TRANSITIONS: LazyCell<Vec<StateTransitionRule>> = LazyCell::new(|| {
        vec![
            // Constructor automatically ignores self-transition rules,
            // so duplicates here are fine
            StateTransitionRule::always(
                StateMatcher::Multiple(_DEFAULT_STATES_PASSIVE.clone()),
                StateMatcher::Multiple(_DEFAULT_STATES.clone()),
            ),
            StateTransitionRule::new(
                StateMatcher::Single(TypeId::of::<Attacking>()),
                StateMatcher::Multiple(_DEFAULT_STATES_PASSIVE.clone()),
                StateTransitionChecker::custom(Box::new(can_transition_from_attacking)),
            ),
        ]
    });

    pub const _DEFAULT_TRANSITIONS_PASSIVE: LazyCell<Vec<StateTransitionRule>> =
        LazyCell::new(|| {
            vec![StateTransitionRule::always(
                StateMatcher::Multiple(_DEFAULT_STATES_PASSIVE.clone()),
                StateMatcher::Multiple(_DEFAULT_STATES_PASSIVE.clone()),
            )]
        });

    fn can_transition_from_attacking(prev: &dyn CharacterState, _: &dyn CharacterState) -> bool {
        if let Some(attacking) = CharacterState::as_any(prev).downcast_ref::<Attacking>()
            && attacking.time_left > 0.0
        {
            false
        } else {
            true
        }
    }

    #[derive(Component, Debug, Clone, PartialEq, Reflect, Default)]
    #[reflect(Component, CharacterState)]
    pub struct Idle;
    impl CharacterStateMarker for Idle {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, CharacterState, MovementState)]
    pub struct Walking;
    impl CharacterStateMarker for Walking {}
    impl MovementState for Walking {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, CharacterState, MovementState)]
    pub struct Running;
    impl CharacterStateMarker for Running {}
    impl MovementState for Running {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, CharacterState, MovementState)]
    pub struct Sprinting;
    impl CharacterStateMarker for Sprinting {}
    impl MovementState for Sprinting {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, CharacterState, TimedState)]
    pub struct Attacking {
        pub time_left: f32,
    }
    impl CharacterStateMarker for Attacking {}
    impl TimedState for Attacking {
        fn time_left(&self) -> f32 {
            self.time_left
        }
        fn set_time(&mut self, time: f32) {
            self.time_left = time;
        }
    }
}

pub fn get_state(
    entity: Entity,
    tracker: &CharacterStateTracker,
    world: &mut World,
) -> Option<Box<dyn CharacterState>> {
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

pub fn is_in_movement_state(
    entity: Entity,
    tracker: &CharacterStateTracker,
    world: &mut World,
) -> bool {
    let registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = registry.read();
    let reg = type_registry.get(tracker.type_id).unwrap();
    let reflect_component = reg.data::<ReflectComponent>().unwrap();

    if let Ok(entity) = world.get_entity(entity)
        && let Some(reflect_data) = reflect_component.reflect(entity)
        && let Some(reflect_movement_state) = reg.data::<ReflectMovementState>()
    {
        reflect_movement_state.get(reflect_data).is_some()
    } else {
        false
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
        transitions: &StateCapabilities,
        new_state: Box<dyn CharacterState>,
        prev_state: Box<dyn CharacterState>,
    ) -> Result<Self, StateTransitionError> {
        transitions.can_transition(prev_state.as_ref(), new_state.as_ref())?;

        Ok(Self {
            entity,
            new_state,
            prev_state,
        })
    }
}

fn on_state_change(event: On<CharacterStateEvent>, mut world: DeferredWorld) {
    let entity = event.entity;

    // We clone these to move them into the command closure
    let new_state = event.new_state.clone_value();
    let prev_type_id = (*event.prev_state).type_id();
    let new_type_id = (*new_state).type_id();

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
            next_reflect_component.insert(&mut entity_mut, new_state.as_reflect(), &type_registry);

            // Update the tracker component
            entity_mut.insert(CharacterStateTracker {
                type_id: new_type_id,
            });
        } else {
            warn!("Failed to update state for entity {}: ", entity);
        }
    });
}

fn update_timed_state(
    time: Res<Time>,
    mut commands: Commands,
    registry: Res<AppTypeRegistry>,
    query: Query<(Entity, &CharacterStateTracker, &StateCapabilities), With<Character>>,
) {
    let delta = time.delta_secs();
    let type_registry = registry.read();

    for (entity, tracker, state_capabilities) in &query {
        // Find the type registration for the current state
        let Some(registration) = type_registry.get(tracker.type_id) else {
            continue;
        };
        let Some(_) = registration.data::<ReflectTimedState>() else {
            continue;
        };
        let Some(_) = registration.data::<ReflectComponent>() else {
            continue;
        };

        let type_id = tracker.type_id;
        let state_capabilities = state_capabilities.clone();

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

                let prev_data = timed_state.box_clone();

                match CharacterStateEvent::try_new(
                    entity,
                    &state_capabilities,
                    Box::new(default_states::Idle),
                    prev_data,
                ) {
                    Ok(event) => {
                        println!(
                            "Transitioning from {:?} to {:?}",
                            event.prev_state, event.new_state
                        );
                        world.commands().trigger(event);
                    }
                    Err(_) => warn!("Failed to transition to Idle state for entity {}", entity),
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
