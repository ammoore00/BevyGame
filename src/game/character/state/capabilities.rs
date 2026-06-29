use crate::game::character::state::states::{DEFAULT_STATES, DEFAULT_TRANSITIONS};
use crate::game::character::state::tracking::ActionState;
use crate::game::character::*;
use std::collections::HashMap;

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