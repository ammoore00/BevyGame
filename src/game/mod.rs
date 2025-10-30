//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;
use physics::movement;

mod animation;
mod grid;
pub mod level;
mod object;
pub mod player;
mod physics;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        grid::plugin,
        level::plugin,
        movement::plugin,
        object::plugin,
        player::plugin,
    ));
}
