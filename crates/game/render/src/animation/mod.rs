mod tracking;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(tracking::plugin);

    app.configure_sets(
        Update,
        (AnimationSystems::Tracking, AnimationSystems::Update)
            .chain(),
    );
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum AnimationSystems {
    /// Track timers
    Tracking,
    /// Update sprites
    Update,
}
