use crate::debug_options::options::{AiStateRes, PathfindStrategyRes, PathsRes};
use crate::debug_options::render::draw::{LineSettings, draw_sphere, draw_world_line};
use crate::debug_options::render::label::{DebugLabel, DebugLabelAttachedTo};
use crate::debug_options::render::palette::{PATH_COLOR, PATH_LINE_THICKNESS, PATH_NODE_RADIUS};
use bevy::prelude::*;
use common::dev_tools::DebugState;
use common::{GameState, Scale, WorldPosition, marker};
use runtime::characters::Character;
use runtime::prelude::{AiState, Player, ReflectPathfindStrategy, Waypoints};
use widgets::theme::palette::{PRIMARY_TEXT, RED_3, SEPIA_6};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_path_render,
            update_pathfind_strategy_label,
            update_state_label,
        ),
    );
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

//------ State ------//

marker!(PathfindStrategyLabel);

fn update_pathfind_strategy_label(world: &mut World) {
    let mut label_query_state = world.query_filtered::<Entity, With<PathfindStrategyLabel>>();
    let label_query = label_query_state.query(world);
    let label_entities = label_query.into_iter().collect::<Vec<_>>();
    
    for label_entity in label_entities {
        world.despawn(label_entity);
    }
    
    if !world.resource::<PathfindStrategyRes>().get() {
        return;
    }
    
    let mut character_query_state = world.query_filtered::<Entity, (With<Character>, Without<Player>)>();
    let character_query = character_query_state.query(world);
    let character_entities = character_query.into_iter().collect::<Vec<_>>();

    for character_entity in character_entities {
        let strategy_components = get_strategy_reflect(world, character_entity);
        
        match strategy_components.len() {
            0 => continue,
            1 => {
                world.spawn((
                    PathfindStrategyLabel,
                    DebugLabel {
                        name: "Pathfinder".to_string(),
                        name_color: PRIMARY_TEXT,

                        text: format!("{}", strategy_components[0]),
                        text_color: SEPIA_6,
                    },
                    DebugLabelAttachedTo(character_entity),
                ));
            }
            _ => {
                world.spawn((
                    PathfindStrategyLabel,
                    DebugLabel {
                        name: "Pathfinder".to_string(),
                        name_color: PRIMARY_TEXT,
                        
                        text: "ERROR: Multiple strategies".to_string(),
                        text_color: RED_3,
                    },
                    DebugLabelAttachedTo(character_entity),
                ));
                error!(
                    "Character {} has multiple pathfinding strategies!",
                    character_entity
                );
            }
        }
    }
}

fn get_strategy_reflect(world: &World, entity: Entity) -> Vec<String> {
    let type_registry = world.resource::<AppTypeRegistry>().read();
    let entity_ref = world.entity(entity);

    let mut strategies = Vec::new();

    for component_id in entity_ref.archetype().components() {
        let component_id = *component_id;

        if let Some(component_info) = world.components().get_info(component_id)
            && let Some(type_id) = component_info.type_id()
            && let Some(registration) = type_registry.get(type_id)
            && registration.data::<ReflectPathfindStrategy>().is_some()
        {
            let name = component_info.name();
            let short_name = name
                .rsplit("::")
                .next()
                .map(|s| s.to_string())
                .unwrap_or(component_info.name().to_string());
            
            strategies.push(short_name.to_string());
        }
    }

    strategies
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
