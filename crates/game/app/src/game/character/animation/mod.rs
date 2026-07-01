use bevy::prelude::*;

mod assets;
mod components;
mod update;

pub use components::{AnimationStateMap, CharacterAnimationTracker};
pub use assets::{AnimationResource, FrameData, AnimationData, AnimationContext};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        assets::plugin,
        update::plugin,
    ));
}