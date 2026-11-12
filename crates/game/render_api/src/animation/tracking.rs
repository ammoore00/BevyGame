use crate::animation::events::{AnimationFinished, AnimationSettings, PlayAnimation, StopAnimation};
use crate::animation::
AnimationSystems;
use assets::resource::{AnimationData, AnimationResource, ResolvedAnimationData};
use bevy::prelude::*;
use data::prelude::{ResourceLocation, SystemRegistry};
use getset::{CopyGetters, Getters};
use std::fmt::{Display, Formatter};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_tracking.in_set(AnimationSystems::Tracking));

    app.add_observer(on_play_animation);
    app.add_observer(on_stop_animation);
}

#[derive(Component, Debug, Clone, Getters, CopyGetters)]
pub struct AnimationTracker {
    /// The handle for the current animation asset.
    #[getset(get = "pub")]
    handle: Handle<AnimationData>,

    /// The current tile sheet index to set the sprite.
    #[getset(get_copy = "pub")]
    current_index: usize,

    /// A cache of the resource location corresponding to the animation asset.
    /// Used only as a comparison for event triggers or for debug display.
    ///
    /// Favor `handle` for all other purposes.
    loc: ResourceLocation<AnimationResource>,

    /// Timer for tracking the current frame.
    frame_timer: Timer,

    /// Counter for tracking the current index relative to the frame data list from the animation.
    /// This is used to calculate the current frame index, but does not itself directly relate to tile sheet indices.
    frame_counter: usize,

    /// Settings set during the initial event trigger controlling behavior of the animation.
    settings: AnimationSettings,
}
impl AnimationTracker {
    /// Creates a new animation tracker component
    ///
    /// # Errors
    /// Returns an error if the animation asset cannot be retrieved from the registry,
    /// if the animation asset is not resolved,
    /// or if the animation does not have any frame data
    fn try_new(
        loc: &ResourceLocation<AnimationResource>,
        registry: &SystemRegistry<AnimationResource>,
        settings: AnimationSettings,
    ) -> Result<Self, AnimationTrackerError> {
        let handle = registry
            .get_handle(loc)
            .ok_or(AnimationTrackerError::AssetRetrieval(loc.clone()))?;

        let animation = registry
            .get_asset_from_handle(&handle)
            .ok_or(AnimationTrackerError::AssetRetrieval(loc.clone()))?
            .as_resolved()
            .ok_or(AnimationTrackerError::AssetNotResolved(loc.clone()))?;

        let first_frame_time = animation
            .frame_data()
            .get_frame_duration(0)
            .ok_or(AnimationTrackerError::InvalidAnimation(
                loc.clone(),
                "Animation has no frame data".to_string(),
            ))?
            .as_secs_f32();
        let frame_timer = Timer::from_seconds(first_frame_time, TimerMode::Repeating);

        let first_index = animation.frame_data().get_frame_index(0).ok_or(
            AnimationTrackerError::InvalidAnimation(
                loc.clone(),
                "Animation has no frame data".to_string(),
            ),
        )?;

        Ok(Self {
            handle,
            current_index: first_index,
            loc: loc.clone(),
            frame_timer,
            frame_counter: 0,
            settings,
        })
    }

    /// Gets the animation data for this tracker.
    pub fn animation<'a>(
        &self,
        registry: &'a SystemRegistry<AnimationResource>,
    ) -> &'a ResolvedAnimationData {
        registry
            .get_asset_from_handle(&self.handle)
            .map(|a| a.as_resolved())
            .flatten()
            .unwrap_or_else(|| {
                // Invariant: AnimationTracker::try_new only constructs trackers with a valid,
                // resolved animation handle. Reaching this means tracker state was somehow invalid.
                unreachable!(
                    "Invalid animation handle stored in tracker for animation: {}",
                    self.loc
                )
            })
    }
}
impl Display for AnimationTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.loc)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
enum AnimationTrackerError {
    #[error("Failed to load animation asset {}", .0)]
    AssetRetrieval(ResourceLocation<AnimationResource>),

    #[error("Requested animation {} is not resolved!", .0)]
    AssetNotResolved(ResourceLocation<AnimationResource>),

    #[error("Requested animation {} is invalid! {}", .0, .1)]
    InvalidAnimation(ResourceLocation<AnimationResource>, String),
}

/// Update the internal state of the animation tracker
fn update_tracking(
    tracker_query: Query<(Entity, &mut AnimationTracker)>,
    registry: SystemRegistry<AnimationResource>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut tracker) in tracker_query {
        tracker.frame_timer.tick(time.delta());

        if tracker.frame_timer.just_finished() {
            let animation = tracker.animation(&registry);

            // Increment the frame counter, looping to the beginning as needed
            tracker.frame_counter += 1;
            if tracker.frame_counter >= animation.frame_data().num_frames() {
                // TODO: Account for repeat setting
                tracker.frame_counter = 0;

                commands.trigger(AnimationFinished::new(
                    entity,
                    tracker.loc.clone(),
                    tracker.settings.repeat,
                ));
            }

            // Update the current frame index
            tracker.current_index = animation.frame_data().frame_index(tracker.frame_counter);

            // If the next frame has a different duration, update the timer
            let next_frame_duration = animation.frame_data().frame_duration(tracker.frame_counter);

            if tracker.frame_timer.duration() != next_frame_duration {
                tracker.frame_timer.set_duration(next_frame_duration);
            }
        }
    }
}

fn on_play_animation(
    event: On<PlayAnimation>,
    tracker_query: Query<&AnimationTracker>,
    animation_registry: SystemRegistry<AnimationResource>,
    mut commands: Commands,
) {
    // If we're already playing an animation, and it's the same animation as the event, and we don't want to restart it, do nothing
    if let Ok(tracker) = tracker_query.get(event.entity)
        && tracker.loc == event.animation
        && !event.settings.restart_if_present
    {
        return;
    }

    // Otherwise, create a new animation tracker
    let tracker =
        match AnimationTracker::try_new(&event.animation, &animation_registry, event.settings) {
            Ok(tracker) => tracker,
            Err(err) => {
                error!("Failed to create animation tracker for: {}", err);
                return;
            }
        };
    commands.entity(event.entity).insert(tracker);
}

fn on_stop_animation(
    event: On<StopAnimation>,
    tracker_query: Query<&AnimationTracker>,
    mut commands: Commands,
) {
    let tracker = match tracker_query.get(event.entity) {
        Ok(tracker) => tracker,
        Err(_) => {
            error!("Called stop animation on an entity that has no animation!");
            return;
        }
    };

    if match event.filter {
        Some(ref filter) => tracker.loc == *filter,
        None => true,
    } {
        commands.entity(event.entity).remove::<AnimationTracker>();
    }
}
