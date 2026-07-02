//! The pause menu.

use crate::theme::widgets;
use crate::theme::widgets::{button, text};
use crate::{menus::Menu, screens::Screen};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use input::gamepad::gamepad_just_pressed;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu.spawn());
    app.add_systems(
        Update,
        go_back.run_if(
            in_state(Menu::Pause).and_then(
                // Exit pause menu if exit buttons (Escape/B/Start by default) are pressed
                input_just_pressed(KeyCode::Escape)
                    .or_else(gamepad_just_pressed(GamepadButton::East))
                    .or_else(gamepad_just_pressed(GamepadButton::Start)),
            ),
        ),
    );
}

fn spawn_pause_menu() -> impl Scene {
    bsn! [
        #PauseMenu
        widgets::ui_root()
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::Pause)
        Children [
            text::header("Game paused"),
            button::with_text("Continue", close_menu),
            button::with_text("Settings", open_settings_menu),
            button::with_text("Quit to Title", quit_to_title),
        ]
    ]
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn close_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn quit_to_title(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
