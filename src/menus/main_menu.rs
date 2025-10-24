//! The main menu (seen on the title screen).

use bevy::input_focus::directional_navigation::DirectionalNavigationMap;
use bevy::input_focus::InputFocus;
use bevy::math::CompassOctant;
use bevy::prelude::*;
use crate::{asset_tracking::ResourceHandles, menus::Menu, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(
    mut directional_nav_map: ResMut<DirectionalNavigationMap>,
    mut input_focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    let ui_root = commands.spawn((
        widget::ui_root("Main Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Main),
    )).id();

    #[cfg(not(target_family = "wasm"))]
    {
        let play_button = commands.spawn(
            widget::button("Play", enter_loading_or_gameplay_screen)
        ).id();
        commands.entity(ui_root).add_child(play_button);

        let settings_button = commands.spawn(
            widget::button("Settings", open_settings_menu)
        ).id();
        commands.entity(ui_root).add_child(settings_button);

        let credits_button = commands.spawn(
            widget::button("Credits", open_credits_menu)
        ).id();
        commands.entity(ui_root).add_child(credits_button);

        let exit_button = commands.spawn(
            widget::button("Exit", exit_app)
        ).id();
        commands.entity(ui_root).add_child(exit_button);

        directional_nav_map.add_looping_edges(&[
            play_button,
            settings_button,
            credits_button,
            exit_button,
        ], CompassOctant::South);

        input_focus.0 = Some(play_button);
    }
    #[cfg(target_family = "wasm")]
    {
        let play_button = commands.spawn(
            widget::button("Play", crate::menus::main_menu::enter_loading_or_gameplay_screen)
        ).id();
        commands.entity(ui_root).add_child(play_button);

        let settings_button = commands.spawn(
            widget::button("Settings", crate::menus::main_menu::open_settings_menu)
        ).id();
        commands.entity(ui_root).add_child(settings_button);

        let credits_button = commands.spawn(
            widget::button("Credits", crate::menus::main_menu::open_credits_menu)
        ).id();
        commands.entity(ui_root).add_child(credits_button);

        directional_nav_map.add_looping_edges(&[
            play_button,
            settings_button,
            credits_button,
        ], CompassOctant::South);

        input_focus.0 = Some(play_button);
    }
}

fn enter_loading_or_gameplay_screen(
    _: On<Pointer<Click>>,
    resource_handles: Res<ResourceHandles>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if resource_handles.is_all_done() {
        next_screen.set(Screen::Gameplay);
    } else {
        next_screen.set(Screen::Loading);
    }
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: On<Pointer<Click>>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
