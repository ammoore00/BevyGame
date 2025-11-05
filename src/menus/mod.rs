//! The game's menus and transitions between them.

mod credits;
mod gamepad_navigation;
mod hud;
mod main_menu;
mod pause;
mod settings;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();

    app.add_plugins((
        credits::plugin,
        hud::plugin,
        main_menu::plugin,
        settings::plugin,
        pause::plugin,
        gamepad_navigation::plugin,
    ));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Menu {
    #[default]
    None,
    Main,
    Credits,
    Settings,
    Pause,
}
