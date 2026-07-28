use bevy::prelude::*;

mod components;
mod update;

pub use components::{AnimationStateMap, CharacterAnimationTracker};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((update::plugin,));
}
