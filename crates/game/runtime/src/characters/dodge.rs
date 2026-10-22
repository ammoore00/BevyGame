use bevy::prelude::*;
use common::Facing;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_try_dodge);
}

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
pub struct TryDodgeEvent {
    entity: Entity,
    facing: Facing,
}

fn on_try_dodge(
    event: On<TryDodgeEvent>,
    mut commands: Commands,
) {
    info!("Try Dodge!");
}
