use bevy::prelude::*;

pub mod character;
pub mod level;
mod object;
mod particle;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        character::plugin,
        level::plugin,
        particle::plugin,
        object::plugin,
    ));
}
