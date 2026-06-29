use crate::prelude::*;
use crate::game::character::animation::assets::AnimationData;
use crate::game::character::animation::{assets, AnimationContext, AnimationStateMap, CharacterAnimationTracker};
use crate::game::character::state::states::Attacking;
use crate::game::character::state::tracking::ActionStateTracker;
use crate::game::character::Facing;
use tracing::warn;
use crate::screens::Screen;
use game_data::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (update_animation_state, update_animation_atlas)
                .chain()
                .in_set(AppSystems::Respond),
        )
            .run_if(in_state(Screen::Gameplay))
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

fn update_animation_state(
    mut query: Query<(
        &ActionStateTracker,
        &Facing,
        &AnimationStateMap,
        &mut CharacterAnimationTracker,
        Option<&Attacking>,
    )>,
    attack_context: SystemRegistry<AttackResource>,
    animation_context: AnimationContext,
) {
    for (
        state_tracker,
        facing,
        animation_state_map,
        mut animation_tracker,
        attacking_state
    ) in &mut query {
        animation_tracker.facing = *facing;

        let Some(animation_handle) = assets::get_animation_handle(
            state_tracker,
            animation_state_map,
            attacking_state,
            &attack_context,
            &animation_context,
        ) else {
            warn!("Failed to get animation handle!");
            return;
        };

        // Update animation tracker state if the animation has changed
        animation_tracker.current_animation = animation_handle.clone();
        let animation = animation_context.get_data_from_handle(animation_handle.clone()).unwrap();

        let interval = animation.frame_data().frame_duration(0).unwrap();

        if animation_tracker.prev_animation != animation_tracker.current_animation {
            animation_tracker.timer = Timer::new(interval, TimerMode::Repeating);
            animation_tracker.frame = 0;
        }

        animation_tracker.prev_animation = animation_handle;
    }
}

fn update_animation_atlas(
    mut query: Query<(
        &ActionStateTracker,
        &CharacterAnimationTracker,
        &AnimationStateMap,
        &mut Sprite,
        Option<&Attacking>,
    )>,
    attack_context: SystemRegistry<AttackResource>,
    animation_context: AnimationContext,
) {
    for (
        state_tracker,
        animation_tracker,
        animation_state_map,
        mut sprite,
        attacking_state
    ) in &mut query {
        let Some(animation_handle) = assets::get_animation_handle(
            state_tracker,
            animation_state_map,
            attacking_state,
            &attack_context,
            &animation_context,
        ) else {
            warn!("Failed to get animation handle!");
            return;
        };

        let animation = animation_context.get_data_from_handle(animation_handle).unwrap();

        sprite.image = animation.image().clone();

        let mut atlas = animation.atlas().clone();
        // Calculate index: (Direction Row * Frames per row) + Current Frame
        atlas.index = animation_tracker.get_atlas_index(animation_context.resolved_assets());
        sprite.texture_atlas = Some(atlas);
    }
}