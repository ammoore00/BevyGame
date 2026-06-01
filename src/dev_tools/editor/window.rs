use crate::menus::font::FontBuilder;
use crate::screens::Screen;
use crate::theme::widget;
use crate::theme::widget::ButtonAssets;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Editor), spawn_editor_window);
}

#[derive(Component, Debug, Clone)]
struct EditorUiRoot;

fn spawn_editor_window(
    button_assets: Res<ButtonAssets>,
    font_builder: FontBuilder,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands
) {
    let editor = commands
        .spawn((
            EditorUiRoot,
            widget::ui_root("Editor"),
            GlobalZIndex(1),
            DespawnOnExit(Screen::Editor),
        )).id();

    let menu_bar = spawn_menu_bar(&button_assets, &font_builder, &mut texture_atlas_layouts, commands.reborrow());
    commands.entity(editor).add_child(menu_bar);

    let editor_content = spawn_editor_content(&button_assets, &font_builder, &mut texture_atlas_layouts, commands.reborrow());
    commands.entity(editor).add_child(editor_content);
}

#[derive(Component, Debug, Clone)]
struct MenuBarRoot;

const MENU_BAR_HEIGHT: usize = 50;

fn spawn_menu_bar(
    button_assets: &ButtonAssets,
    font_builder: &FontBuilder,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    mut commands: Commands
) -> Entity {
    let menu_bar = commands.spawn((
        MenuBarRoot,
        Node {
            flex_direction: FlexDirection::Row,

            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            right: px(0),
            height: px(MENU_BAR_HEIGHT),

            ..Default::default()
        },
    )).id();

    let file_button = commands.spawn(widget::sized_button(
        button_assets,
        texture_atlas_layouts,
        "File",
        px(100),
        percent(100),
        24.,
        font_builder,
        file_button_clicked,
    )).id();
    commands.entity(menu_bar).add_child(file_button);

    let edit_button = commands.spawn(widget::sized_button(
        button_assets,
        texture_atlas_layouts,
        "Edit",
        px(100),
        percent(100),
        24.,
        font_builder,
        edit_button_clicked,
    )).id();
    commands.entity(menu_bar).add_child(edit_button);

    menu_bar
}

fn file_button_clicked(
    _: On<Pointer<Click>>,
) {}

fn edit_button_clicked(
    _: On<Pointer<Click>>,
) {}

#[derive(Component, Debug, Clone)]
struct EditorContentRoot;

fn spawn_editor_content(
    button_assets: &ButtonAssets,
    font_builder: &FontBuilder,
    mut texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    mut commands: Commands
) -> Entity {
    let editor_content = commands.spawn((
        EditorContentRoot,
        Node {
            position_type: PositionType::Absolute,
            top: px(MENU_BAR_HEIGHT),
            left: px(0),
            right: px(0),
            bottom: px(0),

            ..Default::default()
        },
    )).id();

    editor_content
}