//! A loading screen during which game resource are loaded if necessary.
//! This reduces stuttering, especially for audio on Wasm.

use bevy::prelude::*;

use crate::{asset_tracking::ResourceHandles, screens::Screen};
use widgets::text;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading(&Screen::Gameplay)), spawn_gameplay_loading_screen.spawn());
    #[cfg(feature = "dev")]
    app.add_systems(OnEnter(Screen::Loading(&Screen::Editor)), spawn_editor_loading_screen.spawn());

    app.add_systems(
        Update,
        (
            enter_gameplay_screen.run_if(in_state(Screen::Loading(&Screen::Gameplay)).and_then(all_assets_loaded)),
            #[cfg(feature = "dev")]
            enter_editor_screen.run_if(in_state(Screen::Loading(&Screen::Editor)).and_then(all_assets_loaded)),
        ),
    );
}

fn spawn_gameplay_loading_screen() -> impl Scene {
    bsn! [
        #LoadingScreen
        widgets::background::ui_root()
        DespawnOnExit<Screen>(Screen::Loading({&Screen::Gameplay}))
        Children [text::label("Loading...")]
    ]
}

#[cfg(feature = "dev")]
fn spawn_editor_loading_screen() -> impl Scene {
    bsn! [
        #LoadingScreen
        widgets::background::ui_root()
        DespawnOnExit<Screen>(Screen::Loading({&Screen::Editor}))
        Children [text::label("Loading...")]
    ]
}

fn enter_gameplay_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Gameplay);
}

#[cfg(feature = "dev")]
fn enter_editor_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Editor);
}

fn all_assets_loaded(resource_handles: Res<ResourceHandles>) -> bool {
    resource_handles.is_all_done()
}
