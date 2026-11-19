use crate::characters::animation::{AnimationStateMap, CharacterAnimationTracker};
use crate::characters::attack::CurrentAttack;
use crate::characters::state::ActionStateTracker;
use assets::resource::AnimationResource;
use assets::resource::characters::CharacterAnimationSet;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use common::{AppSystems, Facing, GameplaySystems, PausableSystems};
use data::prelude::*;
use render_api::animation::{AnimationSettings, PlayAnimation};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_animation_state
            .in_set(AppSystems::Respond)
            .in_set(GameplaySystems)
            .in_set(PausableSystems),
    );
}

#[derive(QueryData)]
#[query_data(mutable)]
struct AnimationQueryData {
    entity: Entity,
    action_tracker: &'static ActionStateTracker,
    animation_tracker: &'static mut CharacterAnimationTracker,
    animation_state_map: &'static AnimationStateMap,
    facing: &'static Facing,
    current_attack: Option<&'static CurrentAttack>,
}

fn update_animation_state(
    animation_query: Query<AnimationQueryData>,
    mut commands: Commands,
) {
    for animation_data in animation_query {
        let AnimationQueryDataItem {
            entity,
            action_tracker,
            mut animation_tracker,
            animation_state_map,
            facing,
            current_attack,
        } = animation_data;

        let Some(animation_set) =
            get_animation_set(action_tracker, animation_state_map, current_attack)
        else {
            error!("Failed to get animation handle!");
            return;
        };

        // If the animation set has not changed, and we still have the same facing, do nothing
        let set_unchanged = animation_tracker.current_animation == animation_set;
        if set_unchanged && animation_tracker.last_facing == Some(*facing) {
            continue;
        }

        // Update the tracker
        animation_tracker.current_animation = animation_set.clone();
        animation_tracker.last_facing = Some(*facing);
        
        commands.trigger(PlayAnimation {
            entity,
            animation: animation_set.get(*facing).clone(),
            settings: AnimationSettings {
                // If the animation set did not change, but the facing did,
                // start the new animation from the current frame of the previous animation
                continue_progress: set_unchanged,
                ..AnimationSettings::default_gameplay()
            },
        })
    }
}

pub fn get_animation_set(
    state_tracker: &ActionStateTracker,
    animation_state_map: &AnimationStateMap,
    attacking_state: Option<&CurrentAttack>,
) -> Option<CharacterAnimationSet> {
    if let Some(attack) = attacking_state
        && let Some(animation_handle) = animation_state_map
            .attack_animations()
            .get(attack.loc())
            .cloned()
    {
        Some(animation_handle)
    } else if let Some(animation_handle) = animation_state_map
        .standard_animations()
        .get(&state_tracker.state_type_id())
        .cloned()
    {
        Some(animation_handle)
    } else {
        error!("Could not find animation data for state!");
        None
    }
}
