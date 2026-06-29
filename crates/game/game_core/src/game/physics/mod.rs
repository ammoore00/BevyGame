use bevy::app::App;

pub mod components;
pub mod movement;
pub mod math;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((components::plugin, movement::plugin));
}
