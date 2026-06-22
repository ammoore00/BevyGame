//! A loading screen during which game assets are loaded if necessary.
//! This reduces stuttering, especially for audio on Wasm.

use bevy::prelude::*;

use crate::theme::widgets::text::FontBuilder;
use crate::{asset_tracking::ResourceHandles, screens::Screen, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading(&Screen::Gameplay)), spawn_gameplay_loading_screen);
    #[cfg(feature = "dev")]
    app.add_systems(OnEnter(Screen::Loading(&Screen::Editor)), spawn_editor_loading_screen);

    app.add_systems(
        Update,
        (
            enter_gameplay_screen.run_if(in_state(Screen::Loading(&Screen::Gameplay)).and(all_assets_loaded)),
            #[cfg(feature = "dev")]
            enter_editor_screen.run_if(in_state(Screen::Loading(&Screen::Editor)).and(all_assets_loaded)),
        ),
    );
}

fn spawn_gameplay_loading_screen(
    font_builder: FontBuilder,
    mut commands: Commands
) {
    commands.spawn((
        widget_old::ui_root("Loading Screen"),
        DespawnOnExit(Screen::Loading(&Screen::Gameplay)),
        children![widget_old::label_old("Loading...", &font_builder)],
    ));
}

#[cfg(feature = "dev")]
fn spawn_editor_loading_screen(
    font_builder: FontBuilder,
    mut commands: Commands
) {
    commands.spawn((
        widget_old::ui_root("Loading Screen"),
        DespawnOnExit(Screen::Loading(&Screen::Editor)),
        children![widget_old::label_old("Loading...", &font_builder)],
    ));
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
