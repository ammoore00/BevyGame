//! The credits menu.

use crate::gamepad::gamepad_just_pressed;
use crate::theme::widgets;
use crate::theme::widgets::{button, text};
use crate::{audio::music, menus::Menu};
use assets::resource::AudioResource;
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use data::prelude::loc;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu.spawn());
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Credits).and_then(
            input_just_pressed(KeyCode::Escape).or_else(gamepad_just_pressed(GamepadButton::East)),
        )),
    );

    app.add_systems(OnEnter(Menu::Credits), start_credits_music);
}

fn spawn_credits_menu() -> impl Scene {
    bsn! [
        #CreditsMenu
        widgets::scrollable_ui_root()
        GlobalZIndex(2)
        DespawnOnExit<Menu>(Menu::Credits)
        Children [
            text::header("Created by"),
            created_by(),
            text::header("Assets"),
            assets(),
            text::header("License"),
            license(),
            button::with_text("Back", go_back_on_click)
        ]
    ]
    // TODO: Figure out a way to set the input focus
}

macro_rules! grid {
    ($([$label:expr, $text:expr $(,)*]),* $(,)?) => {
        bsn! [
            Node {
                display: Display::Grid,
                row_gap: px(10),
                column_gap: px(30),
                grid_template_columns: RepeatedGridTrack::px::<Vec<RepeatedGridTrack>>(2, 500.0),
            }
            Children [
                $(
                    (
                        text::label($label)
                        TextLayout {
                            justify: Justify::Right
                        }
                    ),
                    (
                        text::label($text)
                        TextLayout {
                            justify: Justify::Left
                        }
                    )
                )*
            ]
        ]
    };
}

fn created_by() -> impl Scene {
    grid!(["The Lady Dawn", "Art, Programming"])
}

fn assets() -> impl Scene {
    grid!(
        ["Button SFX", "CC0 by Jaszunio15"],
        [
            "Music",
            "CC BY 3.0 by Kevin MacLeod\nCC BY 4.0 by Tim Kulig",
        ],
        ["Font", "Open Font License by BoldPixels"],
        ["Character Templates", "ZeggyGames - zegley.itch.io"],
        ["User Interface", "LimeZu - limezu.itch.io"],
        [
            "Bevy Logo",
            "All rights reserved by the Bevy Foundation, permission granted for splash screen use when unmodified",
        ],
    )
}

fn license() -> impl Scene {
    grid!(
        ["Engine Code", "Mozilla Public License 2.0"],
        ["Assets and Game Content", "All Rights Reserved"],
        ["Provisions granted for user generated content", "See LICENSE.md for more information"],
    )
}

fn go_back_on_click(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn start_credits_music(mut commands: Commands) {
    commands.spawn_scene(bsn! [
        Name::new("Credits Music")
        DespawnOnExit<Menu>(Menu::Credits)
        music(loc::<AudioResource>("music/monkeys_spinning_monkeys").unwrap())
    ]);
}
