use crate::debug_options::options::{AiStateRes, PathsRes};
use crate::debug_options::render::draw::{LineSettings, draw_sphere, draw_world_line};
use crate::debug_options::render::palette::{PATH_COLOR, PATH_LINE_THICKNESS, PATH_NODE_RADIUS};
use bevy::prelude::*;
use common::dev_tools::DebugState;
use common::{GameState, Scale, WorldPosition, marker};
use runtime::prelude::{AiState, Waypoints};
use widgets::theme::palette::{PRIMARY_TEXT, SEPIA_6};
use crate::debug_options::render::label::{DebugLabel, DebugLabelAttachedTo};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (update_state_label, update_path_render));
}

//------ State ------//

marker!(AiStateLabel);

fn update_state_label(
    character_query: Query<(Entity, &AiState)>,
    label_query: Query<Entity, With<AiStateLabel>>,
    should_render_ai_state: Res<AiStateRes>,
    mut commands: Commands,
) {
    for entity in label_query.iter() {
        commands.entity(entity).despawn();
    }

    if !should_render_ai_state.get() {
        return;
    }

    for (character_entity, ai_state) in character_query {
        commands.spawn((
            AiStateLabel,
            DebugLabel {
                name: "AI State".to_string(),
                name_color: PRIMARY_TEXT,

                text: format!("{:?}", ai_state.current()),
                text_color: SEPIA_6,
            },
            DebugLabelAttachedTo(character_entity),
        ));
    }
}

//------ Paths ------//

marker!(PathRender);

fn update_path_render(
    waypoints_query: Query<(&Waypoints, &WorldPosition)>,
    render_query: Query<Entity, With<PathRender>>,
    path_res: Res<PathsRes>,
    scale: Res<Scale>,
    mut commands: Commands,
) {
    for entity in render_query.iter() {
        commands.entity(entity).despawn();
    }

    if !path_res.get() {
        return;
    }

    for (waypoints, pos) in waypoints_query {
        let mut prev_pos = pos.0 + (Vec3::NEG_Y * 0.5);

        for node in waypoints.get_remaining_path() {
            let pos = *node + (Vec3::Y * 0.5);

            let settings = LineSettings {
                color: PATH_COLOR,
                thickness: PATH_LINE_THICKNESS,
            };

            draw_sphere(pos, PATH_NODE_RADIUS, settings, scale.0)
                .into_iter()
                .for_each(|line| {
                    commands.spawn((path_bundle(), line));
                });

            if prev_pos != pos {
                commands.spawn((
                    path_bundle(),
                    draw_world_line(prev_pos, pos, settings, scale.0),
                ));
            }

            prev_pos = pos;
        }
    }
}

fn path_bundle() -> impl Bundle {
    (PathRender, DespawnOnExit(GameState::Gameplay))
}
