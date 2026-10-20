use crate::characters::npc::ai::AiSystems;
use crate::prelude::Player;
use assets::resource::characters::{AiHearingParams, AiSightParams};
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use common::{WorldCoords, WorldPosition};
use getset::Getters;
use std::collections::HashMap;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_sensed_targets.in_set(AiSystems::Sense));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SenseKind {
    Sight,
}

#[derive(Component, Debug, Clone, Default, Getters)]
pub struct SensedTargets {
    #[getset(get = "pub")]
    targets: HashMap<Entity, SenseKind>,
}

#[derive(QueryData)]
struct Senses {
    sight: Option<&'static AiSightParams>,
    hearing: Option<&'static AiHearingParams>,
}

#[derive(QueryData)]
struct TargetData {
    entity: Entity,
    pos: &'static WorldPosition,
}

type TargetQuery<'w, 's> = Query<'w, 's, TargetData, With<Player>>;

// TODO: Expand target filtering
// TODO: Add sense modifiers (invisibility, etc)
// TODO: Add line of sight checks
// TODO: Add memory and timeouts
fn update_sensed_targets(
    senses_query: Query<(Entity, &WorldPosition, Senses, Option<&mut SensedTargets>)>,
    sensible_targets: TargetQuery,
    mut commands: Commands,
) {
    for (entity, pos, senses, mut target_list) in senses_query {
        if let Some(ref mut existing) = target_list {
            existing.targets.clear();
        }

        let mut default_val = None;
        let target_list_ref = match target_list {
            Some(ref mut target_list) => target_list.as_mut(),
            None => {
                default_val = Some(SensedTargets::default());
                // Unwrap is safe because we just initialized a Some value
                default_val.as_mut().unwrap()
            }
        };
        
        // TODO: Add hearing-based detection
        
        if let Some(sight) = senses.sight {
            let sight_targets = get_sight_targets(pos.0, sight, sensible_targets);
            sight_targets.into_iter().for_each(|target| {
                target_list_ref.targets.insert(target, SenseKind::Sight);
            });
        }

        if target_list.is_none() {
            // Unwrap is safe because a value is always initialized above when `target_list` is None
            commands.entity(entity).insert(default_val.unwrap());
        }
    }
}

fn get_sight_targets(
    pos: WorldCoords,
    params: &AiSightParams,
    targets: TargetQuery,
) -> Vec<Entity> {
    let mut sensed = Vec::new();

    for target in targets {
        // TODO: Account for sight angle and line of sight
        if target.pos.0.distance(*pos) <= params.range() {
            sensed.push(target.entity);
        }
    }

    sensed
}
