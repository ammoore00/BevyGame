use crate::characters::state::TrySetStateEvent;
use assets::action_states::Dodging;
use bevy::prelude::*;
use common::Facing;
use std::time::Duration;
use physics::{ApplyImpulse, Impulse};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_try_dodge);
    app.add_observer(on_valid_dodge);
}

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
pub struct TryDodgeEvent {
    entity: Entity,
    facing: Facing,
}

const DODGE_DURATION: Duration = Duration::from_millis(11 * 35);

fn on_try_dodge(event: On<TryDodgeEvent>, mut commands: Commands) {
    let facing = event.facing;

    commands.trigger(
        TrySetStateEvent::new(event.entity, Box::new(Dodging::new(DODGE_DURATION))).with_callback(
            move |entity, mut commands: Commands, result: Result<_, _>| {
                if result.is_ok() {
                    commands.trigger(ValidDodgeEvent::new(entity, facing));
                }
            },
        ),
    );
}

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
struct ValidDodgeEvent {
    entity: Entity,
    facing: Facing,
}

const DODGE_IMPULSE: f32 = 20.0;

fn on_valid_dodge(event: On<ValidDodgeEvent>, mut commands: Commands) {
    commands.trigger(ApplyImpulse::new(event.entity, Impulse(Vec3::from(event.facing) * DODGE_IMPULSE)))
}
