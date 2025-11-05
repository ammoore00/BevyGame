//! The settings menu.
//!
//! Additional settings and accessibility options should go here.

use crate::gamepad::gamepad_just_pressed;
use crate::theme::widget;
use crate::{menus::Menu, screens::Screen};
use bevy::input_focus::InputFocus;
use bevy::input_focus::directional_navigation::DirectionalNavigationMap;
use bevy::{audio::Volume, input::common_conditions::input_just_pressed, prelude::*};
use crate::theme::widget::ButtonAssets;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Settings), spawn_settings_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Settings).and(
            input_just_pressed(KeyCode::Escape).or(gamepad_just_pressed(GamepadButton::East)),
        )),
    );

    app.add_systems(
        Update,
        update_global_volume_label.run_if(in_state(Menu::Settings)),
    );
}

fn spawn_settings_menu(
    button_assets: Res<ButtonAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    directional_nav_map: ResMut<DirectionalNavigationMap>,
    mut input_focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    let grid = settings_grid(&button_assets,
                             &mut texture_atlas_layouts, directional_nav_map, &mut commands);

    let ui_root = commands
        .spawn((
            widget::ui_root("Settings Menu"),
            GlobalZIndex(2),
            DespawnOnExit(Menu::Settings),
            children![widget::header("Settings"),],
        ))
        .id();

    commands.entity(ui_root).add_child(grid);

    let back_button = commands
        .spawn(widget::button(&button_assets,
                              &mut texture_atlas_layouts, "Back", go_back_on_click))
        .id();
    commands.entity(ui_root).add_child(back_button);

    input_focus.0 = Some(back_button);
}

fn settings_grid(
    button_assets: &ButtonAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    directional_nav_map: ResMut<DirectionalNavigationMap>,
    commands: &mut Commands,
) -> Entity {
    let volume_widget = global_volume_widget(button_assets,
                                             texture_atlas_layouts, directional_nav_map, commands);

    let ui_root = commands
        .spawn((
            Name::new("Settings Grid"),
            Node {
                display: Display::Grid,
                row_gap: px(10),
                column_gap: px(30),
                grid_template_columns: RepeatedGridTrack::px(2, 400.0),
                ..default()
            },
            children![(
                widget::label("Master Volume"),
                Node {
                    justify_self: JustifySelf::End,
                    ..default()
                }
            ),],
        ))
        .id();

    commands.entity(ui_root).add_child(volume_widget);

    ui_root
}

fn global_volume_widget(
    button_assets: &ButtonAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    _directional_nav_map: ResMut<DirectionalNavigationMap>,
    commands: &mut Commands,
) -> Entity {
    let ui_root = commands
        .spawn((
            Name::new("Global Volume Widget"),
            Node {
                justify_self: JustifySelf::Start,
                ..default()
            },
        ))
        .id();

    let minus_button = commands
        .spawn(widget::button_small(
            &button_assets,
            texture_atlas_layouts,
            "-",
            lower_global_volume,
        ))
        .id();
    commands.entity(ui_root).add_child(minus_button);

    let current_volume_display = commands
        .spawn((
            Name::new("Current Volume"),
            Node {
                padding: UiRect::horizontal(px(10)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![(widget::label(""), GlobalVolumeLabel)],
        ))
        .id();
    commands.entity(ui_root).add_child(current_volume_display);

    let plus_button = commands
        .spawn(widget::button_small(&button_assets,
                                    texture_atlas_layouts, "+", raise_global_volume))
        .id();
    commands.entity(ui_root).add_child(plus_button);

    // TODO: Proper nav grid for buttons

    ui_root
}

const MIN_VOLUME: f32 = 0.0;
const MAX_VOLUME: f32 = 3.0;

fn lower_global_volume(_: On<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() - 0.1).max(MIN_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

fn raise_global_volume(_: On<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() + 0.1).min(MAX_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct GlobalVolumeLabel;

fn update_global_volume_label(
    global_volume: Res<GlobalVolume>,
    mut label: Single<&mut Text, With<GlobalVolumeLabel>>,
) {
    let percent = 100.0 * global_volume.volume.to_linear();
    label.0 = format!("{percent:3.0}%");
}

fn go_back_on_click(
    _: On<Pointer<Click>>,
    screen: Res<State<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}

fn go_back(screen: Res<State<Screen>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}
