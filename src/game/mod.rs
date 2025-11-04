//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

mod grid;
pub mod level;
mod object;
mod physics;
mod character;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        character::plugin,
        grid::plugin,
        level::plugin,
        physics::plugin,
        object::plugin,
    ));
}
