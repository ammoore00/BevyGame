use bevy::prelude::*;

pub mod character;
mod grid;
pub mod level;
mod object;
mod physics;
mod particle;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        character::plugin,
        grid::plugin,
        level::plugin,
        particle::plugin,
        physics::plugin,
        object::plugin,
    ));
}
