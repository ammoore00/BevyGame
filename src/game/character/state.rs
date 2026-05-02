use std::any::{Any, TypeId};
use std::fmt::Debug;
use bevy::prelude::*;
use tracing::warn;
use bevy::ecs::world::DeferredWorld;
use state_transitions::{ActionStateCapabilities, StateTransitionError};
use crate::AppSystems;
use crate::game::character::Character;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (update_timed_state,).in_set(AppSystems::Update));
    app.add_observer(on_state_change);

    action_states::register_states(app);
}

pub fn action_state(state: impl ActionState + Component) -> impl Bundle {
    (
        ActionStateTracker {
            type_id: state.type_id(),
        },
        state,
    )
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ActionStateTracker {
    pub(crate) type_id: TypeId,
}

pub trait ActionStateMarker: Reflect + Send + Sync + Debug + 'static {}

#[reflect_trait]
pub trait ActionState: Reflect + Send + Sync + Debug + 'static {
    fn clone_value(&self) -> Box<dyn Reflect>;
    fn box_clone(&self) -> Box<dyn ActionState>;
    fn as_any(&self) -> &dyn Any;
}

impl<T: ActionStateMarker + Clone> ActionState for T {
    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }
    fn box_clone(&self) -> Box<dyn ActionState> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[reflect_trait]
pub trait MovementActionState: ActionState {}

#[reflect_trait]
pub trait TimedActionState: ActionState {
    fn time_left(&self) -> f32;
    fn set_time(&mut self, time: f32);
}

pub mod state_transitions {
    use crate::game::character::*;
    use std::collections::HashMap;
    use crate::game::character::state::ActionState;
    use crate::game::character::state::action_states::{DEFAULT_STATES, DEFAULT_TRANSITIONS};

    #[derive(Component, Debug, Clone)]
    pub struct ActionStateCapabilities {
        transition_graph: HashMap<(TypeId, TypeId), StateTransitionRule>,
    }
    impl ActionStateCapabilities {
        pub fn new(
            allowed_states: Vec<TypeId>,
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
                    if let Some(rule) = DEFAULT_TRANSITIONS
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

        pub fn can_transition(
            &self,
            prev: &dyn ActionState,
            next: &dyn ActionState,
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
    impl Default for ActionStateCapabilities {
        fn default() -> Self {
            Self::new(DEFAULT_STATES.clone())
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

        fn matches(&self, prev: &dyn ActionState, next: &dyn ActionState) -> bool {
            self.prev_matcher.matches(prev) && self.next_matcher.matches(next)
        }

        fn matches_type(&self, prev: TypeId, next: TypeId) -> bool {
            self.prev_matcher.matches_type(prev) && self.next_matcher.matches_type(next)
        }

        fn can_transition(
            &self,
            prev: &dyn ActionState,
            next: &dyn ActionState,
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
        fn matches(&self, state: &dyn ActionState) -> bool {
            self.matches_type(state.type_id())
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
        Custom(Arc<RwLock<
            dyn for<'a> Fn(&'a dyn ActionState, &'a dyn ActionState) -> bool
            + Send + Sync
        >>),
        Never,
    }

    impl StateTransitionChecker {
        pub fn custom(
            function: Box<
                dyn for<'a> Fn(&'a dyn ActionState, &'a dyn ActionState) -> bool
                + Send + Sync,
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
            from: Box<dyn ActionState>,
            to: Box<dyn ActionState>,
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

pub mod action_states {
    use crate::game::character::*;
    use crate::game::character::state::state_transitions::{
        StateMatcher, StateTransitionChecker, StateTransitionRule,
    };
    use super::*;
    use std::cell::LazyCell;
    use crate::game::character::state::{ActionState, ActionStateMarker, MovementActionState, TimedActionState};

    pub(in crate::game::character) fn register_states(app: &mut App) {
        app.register_type::<Idle>();
        app.register_type::<Walking>();
        app.register_type::<Running>();
        app.register_type::<Sprinting>();
        app.register_type::<Attacking>();
    }

    pub const DEFAULT_STATES: LazyCell<Vec<TypeId>> = LazyCell::new(|| {
        vec![
            TypeId::of::<Idle>(),
            TypeId::of::<Walking>(),
            TypeId::of::<Running>(),
            TypeId::of::<Sprinting>(),
            TypeId::of::<Attacking>(),
        ]
    });

    pub const DEFAULT_STATES_NON_ATTACKING: LazyCell<Vec<TypeId>> = LazyCell::new(|| {
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
                StateMatcher::Multiple(DEFAULT_STATES_NON_ATTACKING.clone()),
                StateMatcher::Multiple(DEFAULT_STATES.clone()),
            ),
            StateTransitionRule::new(
                StateMatcher::Single(TypeId::of::<Attacking>()),
                StateMatcher::Multiple(DEFAULT_STATES_NON_ATTACKING.clone()),
                StateTransitionChecker::custom(Box::new(can_transition_from_attacking)),
            ),
        ]
    });

    fn can_transition_from_attacking(prev: &dyn ActionState, _: &dyn ActionState) -> bool {
        if let Some(attacking) = ActionState::as_any(prev).downcast_ref::<Attacking>()
            && attacking.time_left > 0.0
        {
            false
        } else {
            true
        }
    }

    #[derive(Component, Debug, Clone, PartialEq, Reflect, Default)]
    #[reflect(Component, ActionState)]
    pub struct Idle;
    impl ActionStateMarker for Idle {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, ActionState, MovementActionState)]
    pub struct Walking;
    impl ActionStateMarker for Walking {}
    impl MovementActionState for Walking {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, ActionState, MovementActionState)]
    pub struct Running;
    impl ActionStateMarker for Running {}
    impl MovementActionState for Running {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, ActionState, MovementActionState)]
    pub struct Sprinting;
    impl ActionStateMarker for Sprinting {}
    impl MovementActionState for Sprinting {}

    #[derive(Component, Debug, Clone, Reflect, Default)]
    #[reflect(Component, ActionState, TimedActionState)]
    pub struct Attacking {
        pub time_left: f32,
    }
    impl ActionStateMarker for Attacking {}
    impl TimedActionState for Attacking {
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
    tracker: &ActionStateTracker,
    world: &mut World,
) -> Option<Box<dyn ActionState>> {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    let reg = type_registry.get(tracker.type_id).unwrap();
    let reflect_component = reg.data::<ReflectComponent>().unwrap();
    let reflect_state = reg.data::<ReflectActionState>().unwrap();

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
    tracker: &ActionStateTracker,
    world: &mut World,
) -> bool {
    let registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = registry.read();
    let reg = type_registry.get(tracker.type_id).unwrap();
    let reflect_component = reg.data::<ReflectComponent>().unwrap();

    if let Ok(entity) = world.get_entity(entity)
        && let Some(reflect_data) = reflect_component.reflect(entity)
        && let Some(reflect_movement_state) = reg.data::<ReflectMovementActionState>()
    {
        reflect_movement_state.get(reflect_data).is_some()
    } else {
        false
    }
}

#[derive(EntityEvent, Debug)]
pub struct CharacterStateEvent {
    entity: Entity,
    new_state: Box<dyn ActionState>,
    prev_state: Box<dyn ActionState>,
}

impl CharacterStateEvent {
    pub fn try_new(
        entity: Entity,
        transitions: &ActionStateCapabilities,
        new_state: Box<dyn ActionState>,
        prev_state: Box<dyn ActionState>,
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
            entity_mut.insert(ActionStateTracker {
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
    query: Query<(Entity, &ActionStateTracker, &ActionStateCapabilities), With<Character>>,
) {
    let delta = time.delta_secs();
    let type_registry = registry.read();

    for (entity, tracker, state_capabilities) in &query {
        // Find the type registration for the current state
        let Some(registration) = type_registry.get(tracker.type_id) else {
            continue;
        };
        let Some(_) = registration.data::<ReflectTimedActionState>() else {
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
            let reflect_timed_state = reg.data::<ReflectTimedActionState>().unwrap();
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
                    Box::new(action_states::Idle),
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