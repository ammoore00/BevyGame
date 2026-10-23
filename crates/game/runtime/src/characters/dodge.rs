use crate::characters::state::TrySetStateEvent;
use assets::action_states::Dodging;
use bevy::prelude::*;
use common::Facing;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_try_dodge);
}

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
pub struct TryDodgeEvent {
    entity: Entity,
    facing: Facing,
}

const DODGE_DURATION: Duration = Duration::from_millis(11 * 35);

fn on_try_dodge(event: On<TryDodgeEvent>, mut commands: Commands) {
    commands.trigger(
        TrySetStateEvent::new(event.entity, Box::new(Dodging::new(DODGE_DURATION))).with_callback(
            move |entity, mut commands: Commands, result: Result<_, _>| {
                if result.is_ok() {
                    commands.trigger(ValidDodgeEvent::new(entity));
                }
            },
        ),
    );
}

#[derive(EntityEvent, Debug, Clone)]
pub struct ValidDodgeEvent {
    entity: Entity,
}
impl ValidDodgeEvent {
    fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
