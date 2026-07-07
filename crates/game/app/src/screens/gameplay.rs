//! The screen state for the main gameplay.

use crate::{menus::Menu, screens::Screen};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use common::{GameState, GameplaySystems, Pause};
use input::gamepad::gamepad_just_pressed;
use runtime::{ResetLevelEvent, SpawnLevelEvent};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (set_game_state, spawn_level).chain()
    );
    app.add_systems(
        OnExit(Screen::Gameplay),
        (reset_level, set_menu_state).chain()
    );

    // Toggle pause on key press.
    app.add_systems(
        Update,
        (
            (pause, spawn_pause_overlay, open_pause_menu)
                .in_set(GameplaySystems)
                .run_if(
                    in_state(Menu::None)
                        .and_then(input_just_pressed(KeyCode::KeyP))
                        .or_else(input_just_pressed(KeyCode::Escape))
                        .or_else(gamepad_just_pressed(GamepadButton::Start)),
                ),
            close_menu
                .in_set(GameplaySystems)
                .run_if(
                    not(in_state(Menu::None))
                    .and_then(input_just_pressed(KeyCode::KeyP)),
                ),
        ),
    );
    app.add_systems(OnExit(Screen::Gameplay), (close_menu, unpause));
    app.add_systems(
        OnEnter(Menu::None),
        unpause.in_set(GameplaySystems),
    );
}

fn spawn_level(mut commands: Commands) {
    commands.trigger(SpawnLevelEvent);
}

fn reset_level(mut commands: Commands) {
    commands.trigger(ResetLevelEvent);
}

fn unpause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause::Unpaused);
}

fn pause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause::Paused);
}

fn spawn_pause_overlay(mut commands: Commands) {
    commands.spawn((
        Name::new("Pause Overlay"),
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(1),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        DespawnOnExit(Pause::Paused),
    ));
}

fn open_pause_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Pause);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn set_game_state(mut next_game_state: ResMut<NextState<GameState>>) {
    next_game_state.set(GameState::Gameplay)
}

fn set_menu_state(mut next_game_state: ResMut<NextState<GameState>>) {
    next_game_state.set(GameState::Menu)
}