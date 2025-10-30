use bevy::app::App;

pub mod components;
pub mod movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        movement::plugin,
    ));
}