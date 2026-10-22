use crate::resource::characters::AttackResource;
use bevy::prelude::*;
use data::prelude::ResourceLocation;
use getset::{Getters, Setters};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, LazyLock, RwLock};
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    register_states(app);
}

pub(super) fn register_states(app: &mut App) {
    app.register_type::<Idle>();
    app.register_type::<Walking>();
    app.register_type::<Running>();
    app.register_type::<Sprinting>();
    app.register_type::<Attacking>();
    app.register_type::<Dodging>();
}

pub static ALL_STATES: LazyLock<Vec<TypeId>> = LazyLock::new(|| {
    vec![
        TypeId::of::<Idle>(),
        TypeId::of::<Walking>(),
        TypeId::of::<Running>(),
        TypeId::of::<Sprinting>(),
        TypeId::of::<Attacking>(),
        TypeId::of::<Dodging>(),
    ]
});

pub static MOVEMENT_STATES: LazyLock<Vec<TypeId>> = LazyLock::new(|| {
    vec![
        TypeId::of::<Idle>(),
        TypeId::of::<Walking>(),
        TypeId::of::<Running>(),
        TypeId::of::<Sprinting>(),
    ]
});

pub trait ActionStateMarker: Reflect + Send + Sync + Debug + 'static {
    fn is_movement() -> bool;
}

#[reflect_trait]
pub trait ActionState: Reflect + Send + Sync + Debug + 'static {
    fn clone_value(&self) -> Box<dyn Reflect>;
    fn box_clone(&self) -> Box<dyn ActionState>;
    fn as_any(&self) -> &dyn Any;
    fn is_movement(&self) -> bool;
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
    fn is_movement(&self) -> bool {
        T::is_movement()
    }
}

#[reflect_trait]
pub trait MovementActionState: ActionState {}

#[reflect_trait]
pub trait TimedActionState: ActionState {
    fn time_left(&self) -> Duration;
    fn set_time(&mut self, time: Duration);
}

#[derive(Component, Debug, Clone, PartialEq, Reflect, Default)]
#[reflect(Component, ActionState)]
pub struct Idle;
impl ActionStateMarker for Idle {
    fn is_movement() -> bool {
        false
    }
}

#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component, ActionState, MovementActionState)]
pub struct Walking;
impl ActionStateMarker for Walking {
    fn is_movement() -> bool {
        true
    }
}
impl MovementActionState for Walking {}

#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component, ActionState, MovementActionState)]
pub struct Running;
impl ActionStateMarker for Running {
    fn is_movement() -> bool {
        true
    }
}
impl MovementActionState for Running {}

#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component, ActionState, MovementActionState)]
pub struct Sprinting;
impl ActionStateMarker for Sprinting {
    fn is_movement() -> bool {
        true
    }
}
impl MovementActionState for Sprinting {}

#[derive(Component, Debug, Clone, Reflect, Getters, Setters)]
#[reflect(Component, ActionState, TimedActionState)]
pub struct Attacking {
    #[getset(get = "pub")]
    attack: ResourceLocation<AttackResource>,
    #[getset(get = "pub", set = "pub")]
    time_left: Duration,
}
impl Attacking {
    pub fn new(attack: &ResourceLocation<AttackResource>, duration: Duration) -> Self {
        Self {
            attack: attack.clone(),
            time_left: duration,
        }
    }
}
impl ActionStateMarker for Attacking {
    fn is_movement() -> bool {
        false
    }
}
impl TimedActionState for Attacking {
    fn time_left(&self) -> Duration {
        self.time_left
    }
    fn set_time(&mut self, time: Duration) {
        self.time_left = time;
    }
}

#[derive(Component, Debug, Clone, Reflect, Getters, Setters)]
#[reflect(Component, ActionState, TimedActionState)]
pub struct Dodging {
    #[getset(get = "pub", set = "pub")]
    time_left: Duration,
}
impl Dodging {
    pub fn new(duration: Duration) -> Self {
        Self {
            time_left: duration,
        }
    }
}
impl ActionStateMarker for Dodging {
    fn is_movement() -> bool {
        false
    }
}
impl TimedActionState for Dodging {
    fn time_left(&self) -> Duration {
        self.time_left
    }
    fn set_time(&mut self, time: Duration) {
        self.time_left = time;
    }
}

pub static DEFAULT_TRANSITIONS: LazyLock<Vec<StateTransitionRule>> = LazyLock::new(|| {
    vec![
        // Constructor automatically ignores self-transition rules,
        // so duplicates here are fine
        StateTransitionRule::always(
            StateMatcher::Multiple(MOVEMENT_STATES.clone()),
            StateMatcher::Multiple(ALL_STATES.clone()),
        ),
        StateTransitionRule::new(
            StateMatcher::Single(TypeId::of::<Attacking>()),
            StateMatcher::Multiple(MOVEMENT_STATES.clone()),
            Box::new(timed_state_transition_checker::<Attacking>),
        ),
        StateTransitionRule::new(
            StateMatcher::Single(TypeId::of::<Dodging>()),
            StateMatcher::Multiple(MOVEMENT_STATES.clone()),
            Box::new(timed_state_transition_checker::<Dodging>),
        ),
    ]
});

fn timed_state_transition_checker<T: TimedActionState>(prev: &dyn ActionState, _: &dyn ActionState) -> bool {
    if let Some(timed_state) = ActionState::as_any(prev).downcast_ref::<T>()
        && timed_state.time_left() > Duration::ZERO
    {
        false
    } else {
        true
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
        transition_checker: Box<StateTransitionFn>,
    ) -> Self {
        Self {
            prev_matcher,
            next_matcher,
            transition_checker: StateTransitionChecker::custom(transition_checker),
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

type StateTransitionFn =
    dyn for<'a> Fn(&'a dyn ActionState, &'a dyn ActionState) -> bool + Send + Sync;

#[derive(Clone)]
pub enum StateTransitionChecker {
    Always,
    Custom(Arc<RwLock<StateTransitionFn>>),
    Never,
}

impl StateTransitionChecker {
    pub fn custom(function: Box<StateTransitionFn>) -> Self {
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

impl std::fmt::Display for InvalidTransitionReason {
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

#[derive(Component, Debug, Clone)]
pub struct ActionStateCapabilities {
    transition_graph: HashMap<(TypeId, TypeId), StateTransitionRule>,
}
impl ActionStateCapabilities {
    pub fn new(allowed_states: Vec<TypeId>) -> Self {
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
        Self::new(ALL_STATES.clone())
    }
}
