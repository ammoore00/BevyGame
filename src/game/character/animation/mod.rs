mod assets;
mod components;
mod update;

use crate::prelude::*;

pub use components::{AnimationStateMap, CharacterAnimationTracker};
pub use assets::{AnimationResource, FrameData, ResolvedAnimationData, ResolvedAnimationRegistry};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        assets::plugin,
        update::plugin,
    ));
}