use crate::characters::animation::{AnimationStateMap, CharacterAnimationTracker};
use crate::characters::state::ActionStateTracker;
use assets::action_states::Attacking;
use assets::resource::characters::{AttackResource, CharacterAnimationSet};
use assets::resource::{AnimationData, AnimationResource};
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use common::{AppSystems, Facing, GameplaySystems, PausableSystems};
use data::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (update_animation_state, update_animation_atlas)
                .chain()
                .in_set(AppSystems::Respond),
        )
            .in_set(GameplaySystems)
            .in_set(PausableSystems),
    );
}

fn update_animation_timer(
    time: Res<Time>,
    assets: Res<Assets<AnimationData>>,
    mut query: Query<&mut CharacterAnimationTracker>,
) {
    for mut animation in &mut query {
        animation.update_timer(time.delta(), &assets);
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct AnimationQueryData {
    action_tracker: &'static ActionStateTracker,
    animation_tracker: &'static mut CharacterAnimationTracker,
    animation_state_map: &'static AnimationStateMap,
    facing: &'static Facing,
    sprite: &'static mut Sprite,
    attacking: Option<&'static Attacking>,
}

fn update_animation_state(
    animation_query: Query<AnimationQueryData>,
    attack_context: SystemRegistry<AttackResource>,
    animation_context: SystemRegistry<AnimationResource>,
) {
    for animation_data in animation_query {
        let AnimationQueryDataItem {
            action_tracker,
            mut animation_tracker,
            animation_state_map,
            facing,
            attacking,
            ..
        } = animation_data;
        
        animation_tracker.facing = *facing;

        let Some(animation_set) = get_animation_set(
            action_tracker,
            animation_state_map,
            attacking,
            &attack_context,
            &animation_context,
        ) else {
            error!("Failed to get animation handle!");
            return;
        };

        // Update animation tracker state if the animation has changed
        animation_tracker.current_animation = animation_set.clone();
        let animation = animation_context
            .get_asset_from_handle(&animation_set.get(*facing))
            .unwrap();

        let interval = animation
            .resolved()
            .frame_data()
            .get_frame_duration(0)
            .unwrap();

        if animation_tracker.prev_animation != animation_tracker.current_animation {
            animation_tracker.timer = Timer::new(interval, TimerMode::Repeating);
            animation_tracker.frame = 0;
        }

        animation_tracker.prev_animation = animation_set;
    }
}

fn update_animation_atlas(
    animation_query: Query<AnimationQueryData>,
    attack_context: SystemRegistry<AttackResource>,
    animation_context: SystemRegistry<AnimationResource>,
) {
    for animation_data in animation_query
    {
        let AnimationQueryDataItem {
            action_tracker,
            animation_tracker,
            animation_state_map,
            facing,
            mut sprite,
            attacking,
        } = animation_data;
        
        let Some(animation_set) = get_animation_set(
            action_tracker,
            animation_state_map,
            attacking,
            &attack_context,
            &animation_context,
        ) else {
            error!("Failed to get animation handle!");
            return;
        };

        let animation = animation_context
            .get_asset_from_handle(&animation_set.get(*facing))
            .unwrap();

        sprite.image = animation.resolved().image().clone();

        let mut atlas = animation.resolved().atlas().clone();
        // Calculate index: (Direction Row * Frames per row) + Current Frame
        atlas.index = animation_tracker.get_atlas_index(animation_context.assets());
        sprite.texture_atlas = Some(atlas);
    }
}

pub fn get_animation_set(
    state_tracker: &ActionStateTracker,
    animation_state_map: &AnimationStateMap,
    attacking_state: Option<&Attacking>,
    attack_context: &SystemRegistry<AttackResource>,
    animation_context: &SystemRegistry<AnimationResource>,
) -> Option<CharacterAnimationSet> {
    if let Some(attacking_state) = attacking_state {
        let Some(attack) = attack_context.get_asset(attacking_state.attack()) else {
            error!(
                "Could not find attack definition for {}!",
                attacking_state.attack()
            );
            return None;
        };

        // TODO: Fix this code to work with new animation systems
        //Some(animation_handle)
        None
    } else if let Some(animation_handle) = animation_state_map
        .get()
        .get(&state_tracker.state_type_id())
        .cloned()
    {
        Some(animation_handle)
    } else {
        error!("Could not find animation data for state!");
        None
    }
}
