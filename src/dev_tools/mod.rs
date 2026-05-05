//! Development tools for the game. This plugin is only enabled in dev builds.

mod debug_options;
mod physics;
mod debug_menu;

use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use std::fmt::Debug;
use std::ops::Not;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
            debug_options::plugin,
            debug_menu::plugin,
        ));
}