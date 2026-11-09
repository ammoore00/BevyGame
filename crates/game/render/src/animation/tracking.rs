use crate::animation::AnimationSystems;
use assets::resource::characters::{AnimationData, AnimationResource};
use bevy::prelude::*;
use data::prelude::{ResourceLocation, SystemRegistry};
use getset::Getters;
use render_api::animation::{AnimationSettings, PlayAnimation, StopAnimation};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_tracking.in_set(AnimationSystems::Tracking));

    app.add_observer(on_play_animation);
    app.add_observer(on_stop_animation);
}

#[derive(Component, Debug, Clone, Getters)]
pub struct AnimationTracker {
    /// The handle for the current animation asset.
    #[getset(get = "pub")]
    handle: Handle<AnimationData>,
    
    /// The current tile sheet index to set the sprite.
    #[getset(get = "pub")]
    current_index: usize,
    
    /// A cache of the resource location corresponding to the animation asset.
    /// Used only as a comparison for event triggers. Favor `handle` for all other purposes.
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
    /// Returns an error if the animation asset cannot be retrieved from the registry, or if the animation asset is not resolved.
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

        todo!()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
enum AnimationTrackerError {
    #[error("Failed to load animation asset {}", .0)]
    AssetRetrieval(ResourceLocation<AnimationResource>),

    #[error("Requested animation {} is not resolved!", .0)]
    AssetNotResolved(ResourceLocation<AnimationResource>),
}

fn update_tracking(
    tracker_query: Query<&mut AnimationTracker>,
    time: Res<Time>,
) {
    for mut tracker in tracker_query {
        tracker.frame_timer.tick(time.delta());
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
    let tracker = match AnimationTracker::try_new(&event.animation, &animation_registry, event.settings) {
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
