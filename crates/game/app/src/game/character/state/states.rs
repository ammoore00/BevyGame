use crate::game::character::state::capabilities::{
    StateMatcher, StateTransitionChecker, StateTransitionRule,
};
use crate::game::character::state::tracking::{
    ActionState, ActionStateMarker, MovementActionState, ReflectActionState,
    ReflectMovementActionState, ReflectTimedActionState, TimedActionState,
};
use crate::game::character::*;
use data::prelude::*;
use getset::{Getters, Setters};
use std::cell::LazyCell;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    register_states(app);
}

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

#[derive(Component, Debug, Clone, Reflect, Getters, Setters)]
#[reflect(Component, ActionState, TimedActionState)]
pub struct Attacking {
    #[getset(get = "pub")]
    attack: ResourceLocation<AttackResource>,
    #[getset(get = "pub", set = "pub")]
    time_left: f32,
}
impl Attacking {
    pub fn new(attack: &ResourceLocation<AttackResource>, duration: Duration) -> Self {
        Self {
            attack: attack.clone(),
            time_left: duration.as_secs_f32(),
        }
    }
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
