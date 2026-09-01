//! The main menu (seen on the title screen).

use crate::{asset_tracking::ResourceHandles, menus::Menu, screens::Screen};
use bevy::prelude::*;
use assets::AssetLoadState;
use widgets::button;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu.spawn());
}

fn spawn_main_menu() -> impl Scene {
    bsn! [
        #MainMenu
        widgets::background::ui_root()
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::Main)
        Children [
            button::with_text("Play", enter_loading_or_gameplay_screen),
            {
                #[cfg(feature = "dev")]
                bsn! [button::with_text("Editor", enter_loading_or_editor_screen)]
            },
            button::with_text("Settings", open_settings_menu),
            button::with_text("Credits", open_credits_menu),
            {
                #[cfg(not(target_family = "wasm"))]
                bsn! [button::with_text("Exit", exit_app)]
            },
        ]
    ]
}

fn enter_loading_or_gameplay_screen(
    _: On<Pointer<Click>>,
    asset_state: Res<State<AssetLoadState>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if *asset_state == AssetLoadState::Done {
        next_screen.set(Screen::Gameplay);
    } else {
        next_screen.set(Screen::Loading(&Screen::Gameplay));
    }
}

#[cfg(feature = "dev")]
fn enter_loading_or_editor_screen(
    _: On<Pointer<Click>>,
    asset_state: Res<State<AssetLoadState>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if *asset_state == AssetLoadState::Done {
        next_screen.set(Screen::Editor);
    } else {
        next_screen.set(Screen::Loading(&Screen::Editor));
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
