mod tracking;
mod sprite;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((sprite::plugin, tracking::plugin));

    app.configure_sets(
        Update,
        (AnimationSystems::Tracking, AnimationSystems::Sprite)
            .chain(),
    );
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum AnimationSystems {
    /// Track timers
    Tracking,
    /// Update sprites
    Sprite,
}
