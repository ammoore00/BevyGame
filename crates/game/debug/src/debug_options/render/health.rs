use crate::debug_options::options::CharacterHealthRes;
use crate::debug_options::render::label::{DebugLabel, DebugLabelAttachedTo};
use bevy::prelude::*;
use common::dev_tools::DebugState;
use common::marker;
use runtime::prelude::Health;
use widgets::theme::palette::{PRIMARY_TEXT, SEPIA_6};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_health_label);
}

marker!(HealthLabel);

fn update_health_label(
    character_query: Query<(Entity, &Health)>,
    label_query: Query<Entity, With<HealthLabel>>,
    should_render_health: Res<CharacterHealthRes>,
    mut commands: Commands,
) {
    for entity in label_query.iter() {
        commands.entity(entity).despawn();
    }

    if !should_render_health.get() {
        return;
    }

    for (character_entity, health) in character_query {
        commands.spawn((
            HealthLabel,
            DebugLabel {
                name: "Health".to_string(),
                name_color: PRIMARY_TEXT,

                text: health.current.to_string(),
                text_color: SEPIA_6,
            },
            DebugLabelAttachedTo(character_entity),
        ));
    }
}
