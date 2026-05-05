//! The credits menu.

use crate::gamepad::gamepad_just_pressed;
use crate::menus::font::FontBuilder;
use crate::theme::widget::ButtonAssets;
use crate::{asset_tracking::LoadResource, audio::music, menus::Menu, theme::prelude::*};
use bevy::input_focus::InputFocus;
use bevy::{ecs::spawn::SpawnIter, input::common_conditions::input_just_pressed, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Credits).and(
            input_just_pressed(KeyCode::Escape).or(gamepad_just_pressed(GamepadButton::East)),
        )),
    );

    app.load_resource::<CreditsAssets>();
    app.add_systems(OnEnter(Menu::Credits), start_credits_music);
}

fn spawn_credits_menu(
    button_assets: Res<ButtonAssets>,
    font_builder: FontBuilder,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut input_focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    let ui_root = commands
        .spawn((
            widget::scrollable_ui_root("Credits Menu"),
            GlobalZIndex(2),
            DespawnOnExit(Menu::Credits),
            children![
                widget::header("Created by", &font_builder),
                created_by(&font_builder),
                widget::header("Assets", &font_builder),
                assets(&font_builder),
                widget::header("License", &font_builder),
                widget::label("This game is provided under the Mozilla Public License 2.0", &font_builder),
                license(&font_builder),
            ],
        ))
        .id();

    let back_button = commands
        .spawn(widget::button(
            &button_assets,
            &mut texture_atlas_layouts,
            "Back",
            &font_builder,
            go_back_on_click,
        ))
        .id();
    commands.entity(ui_root).add_child(back_button);
    input_focus.0 = Some(back_button);
}

fn created_by(font_builder: &FontBuilder) -> impl Bundle {
    grid(vec![["The Lady Dawn", "Art, Programming"]], font_builder)
}

fn assets(font_builder: &FontBuilder) -> impl Bundle {
    grid(vec![
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
    ], font_builder)
}

fn license(font_builder: &FontBuilder) -> impl Bundle {
    grid(vec![
        [
            "More Information",
            "https://www.mozilla.org/en-US/MPL/2.0/FAQ/",
        ],
        [
            "Full License Text",
            "https://www.mozilla.org/en-US/MPL/2.0/",
        ],
    ], font_builder)
}

fn grid(content: Vec<[&'static str; 2]>, font_builder: &FontBuilder) -> impl Bundle {
    let content = content.into_iter().map(|row| {
        [
            widget::label(row[0], &font_builder),
            widget::label(row[1], &font_builder)
        ]
    })
        .collect::<Vec<_>>();
    
    (
        Name::new("Grid"),
        Node {
            display: Display::Grid,
            row_gap: px(10),
            column_gap: px(30),
            grid_template_columns: RepeatedGridTrack::px(2, 500.0),
            ..default()
        },
        Children::spawn(SpawnIter(content.into_iter().flatten().enumerate().map(
            |(i, text)| {
                (
                    text,
                    Node {
                        justify_self: if i.is_multiple_of(2) {
                            JustifySelf::End
                        } else {
                            JustifySelf::Start
                        },
                        ..default()
                    },
                )
            },
        ))),
    )
}

fn go_back_on_click(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct CreditsAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for CreditsAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("base/audio/music/Monkeys Spinning Monkeys.ogg"),
        }
    }
}

fn start_credits_music(mut commands: Commands, credits_music: Res<CreditsAssets>) {
    commands.spawn((
        Name::new("Credits Music"),
        DespawnOnExit(Menu::Credits),
        music(credits_music.music.clone()),
    ));
}
