use crate::characters::state::TrySetStateEvent;
use assets::action_states::Dodging;
use bevy::prelude::*;
use std::time::Duration;
use getset::CopyGetters;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_try_dodge);
}

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
pub struct TryDodgeEvent {
    entity: Entity,
    direction: Vec3,
}

const DODGE_DURATION: Duration = Duration::from_millis(11 * 35);

fn on_try_dodge(event: On<TryDodgeEvent>, mut commands: Commands) {
    commands.trigger(
        TrySetStateEvent::new(event.entity, Box::new(Dodging::new(DODGE_DURATION, event.direction))).with_callback(
            move |entity, mut commands: Commands, result: Result<_, _>| {
                if result.is_ok() {
                    commands.trigger(DodgeValidated::new(entity));
                }
            },
        ),
    );
}

#[derive(EntityEvent, Debug, Clone, CopyGetters)]
pub struct DodgeValidated {
    #[getset(get_copy = "pub")]
    entity: Entity,
}
impl DodgeValidated {
    fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
