use crate::debug_options::options::CharacterHealthRes;
use bevy::prelude::*;
use common::dev_tools::DebugState;
use common::{Scale, WorldPosition, marker};
use runtime::debug::Health;
use widgets::text;
use widgets::text::LARGE_FONT_SIZE;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_health_render);
}

marker!(HealthRender);

fn update_health_render(
    character_query: Query<(&Health, &WorldPosition)>,
    render_query: Query<Entity, With<HealthRender>>,
    scale: Res<Scale>,
    should_render_health: Res<CharacterHealthRes>,
    mut commands: Commands,
) {
    for entity in render_query.iter() {
        commands.entity(entity).despawn();
    }

    if !should_render_health.get() {
        return;
    }

    for (health, pos) in character_query {
        let pos = pos.0 + Vec3::new(0.1, 1.25, 0.1).into();

        commands.spawn_scene(bsn! [
            HealthRender
            text::world_text(health.current.to_string(), LARGE_FONT_SIZE, Color::srgb(0.9, 0.3, 0.2), pos, *scale)
        ]);
    }
}
