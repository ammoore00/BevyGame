//! The pause menu.

use crate::gamepad::gamepad_just_pressed;
use crate::{menus::Menu, screens::Screen, theme::widget};
use bevy::input_focus::InputFocus;
use bevy::input_focus::directional_navigation::DirectionalNavigationMap;
use bevy::math::CompassOctant;
use bevy::{input::common_conditions::input_just_pressed, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
    app.add_systems(
        Update,
        go_back.run_if(
            in_state(Menu::Pause).and(
                input_just_pressed(KeyCode::Escape)
                    .or(gamepad_just_pressed(GamepadButton::East))
                    .or(gamepad_just_pressed(GamepadButton::Start)),
            ),
        ),
    );
}

fn spawn_pause_menu(
    mut directional_nav_map: ResMut<DirectionalNavigationMap>,
    mut input_focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    let ui_root = commands
        .spawn((
            widget::ui_root("Pause Menu"),
            GlobalZIndex(2),
            DespawnOnExit(Menu::Pause),
            children![widget::header("Game paused")],
        ))
        .id();

    let continue_button = commands.spawn(widget::button("Continue", close_menu)).id();
    commands.entity(ui_root).add_child(continue_button);

    let settings_button = commands
        .spawn(widget::button("Settings", open_settings_menu))
        .id();
    commands.entity(ui_root).add_child(settings_button);

    let quit_button = commands
        .spawn(widget::button("Quit to title", quit_to_title))
        .id();
    commands.entity(ui_root).add_child(quit_button);

    directional_nav_map.add_looping_edges(
        &[continue_button, settings_button, quit_button],
        CompassOctant::South,
    );

    input_focus.0 = Some(continue_button);
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
