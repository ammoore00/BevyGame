use crate::menus::font::FontBuilder;
use crate::screens::Screen;
use crate::theme::widget;
use crate::theme::widget::{UiAssets, UiBackgroundStyle};
use bevy::prelude::*;
use crate::dev_tools::editor::window::menu_bar::{spawn_menu_bar, MENU_BAR_TOTAL_HEIGHT};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Editor), spawn_editor_window);
}

#[derive(Component, Debug, Clone)]
struct EditorUiRoot;

fn spawn_editor_window(
    button_assets: Res<UiAssets>,
    font_builder: FontBuilder,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands
) {
    let editor = commands
        .spawn((
            EditorUiRoot,
            widget::ui_root("Editor"),
            DespawnOnExit(Screen::Editor),
        )).id();

    let editor_content = spawn_editor_content(&button_assets, &font_builder, &mut texture_atlas_layouts, commands.reborrow());
    commands.entity(editor).add_child(editor_content);

    let menu_bar = spawn_menu_bar(&button_assets, &font_builder, &mut texture_atlas_layouts, commands.reborrow());
    commands.entity(editor).add_child(menu_bar);
}

#[derive(Component, Debug, Clone)]
struct EditorContentRoot;

#[derive(Component, Debug, Clone)]
struct EditorLeftPanel;

#[derive(Component, Debug, Clone)]
struct EditorCenterPanel;

#[derive(Component, Debug, Clone)]
struct EditorRightPanel;

#[derive(Component, Debug, Clone)]
struct EditorBottomPanel;

const CENTER_PANEL_HEIGHT: usize = 70;
const LOWER_PANEL_HEIGHT: usize = 100 - CENTER_PANEL_HEIGHT;

const LEFT_PANEL_WIDTH: usize = 25;
const RIGHT_PANEL_WIDTH: usize = 25;
const CENTER_PANEL_WIDTH: usize = 100 - LEFT_PANEL_WIDTH - RIGHT_PANEL_WIDTH;

fn spawn_editor_content(
    ui_assets: &UiAssets,
    font_builder: &FontBuilder,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    mut commands: Commands,
) -> Entity {
    let editor_content = commands.spawn((
        EditorContentRoot,
        Node {
            flex_direction: FlexDirection::Column,

            position_type: PositionType::Absolute,
            top: px(MENU_BAR_TOTAL_HEIGHT),
            left: px(0),
            right: px(0),
            bottom: px(0),

            ..Default::default()
        },
        Pickable::IGNORE,
    )).id();

    let center_columns = commands.spawn((
        Node {
            width: percent(100),
            height: percent(CENTER_PANEL_HEIGHT),

            padding: UiRect::horizontal(px(2)),

            ..Default::default()
        },
        Pickable::IGNORE,
    )).id();
    commands.entity(editor_content).add_child(center_columns);

    const BACKGROUND_BLEED: f32 = 10.0;

    let left_panel = commands.spawn((
        EditorLeftPanel,
        Node {
            position_type: PositionType::Relative,

            width: percent(LEFT_PANEL_WIDTH),
            height: percent(100),

            ..Default::default()
        },
        Pickable::IGNORE,
    )).id();
    commands.entity(center_columns).add_child(left_panel);
    spawn_panel_background(
        left_panel,
        ui_assets,
        texture_atlas_layouts,
        UiBackgroundStyle::Panel,
        BACKGROUND_BLEED,
        BACKGROUND_BLEED,
        &mut commands,
    );

    let center_panel = commands.spawn((
        EditorCenterPanel,
        Node {
            position_type: PositionType::Relative,

            width: percent(CENTER_PANEL_WIDTH),
            height: percent(100),

            ..Default::default()
        },
        Pickable::IGNORE,
    )).id();
    commands.entity(center_columns).add_child(center_panel);

    let right_panel = commands.spawn((
        EditorRightPanel,
        Node {
            position_type: PositionType::Relative,

            width: percent(RIGHT_PANEL_WIDTH),
            height: percent(100),

            ..Default::default()
        },
        Pickable::IGNORE,
    )).id();
    commands.entity(center_columns).add_child(right_panel);
    spawn_panel_background(
        right_panel,
        ui_assets,
        texture_atlas_layouts,
        UiBackgroundStyle::Panel,
        BACKGROUND_BLEED,
        BACKGROUND_BLEED,
        &mut commands,
    );

    let bottom_panel = commands.spawn((
        EditorBottomPanel,
        widget::ui_background(ui_assets, texture_atlas_layouts, UiBackgroundStyle::Main),
        Node {
            width: percent(100),
            height: percent(LOWER_PANEL_HEIGHT),

            ..Default::default()
        },
        Pickable::IGNORE,
    )).id();
    commands.entity(editor_content).add_child(bottom_panel);

    editor_content
}

fn spawn_panel_background(
    parent: Entity,
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    style: UiBackgroundStyle,
    top_bleed: f32,
    bottom_bleed: f32,
    commands: &mut Commands,
) {
    let background = commands.spawn((
        widget::ui_background(ui_assets, texture_atlas_layouts, style),
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(-top_bleed),
            bottom: px(-bottom_bleed),
            ..Default::default()
        },
        Pickable::IGNORE,
    )).id();

    commands.entity(parent).add_child(background);
}

mod menu_bar {
    use super::*;

    #[derive(Component, Debug, Clone)]
    struct MenuBarRoot;

    #[derive(Component, Debug, Clone)]
    struct MenuBarButtonsRoot;


    pub(super) const MENU_BUTTON_PER_CHAR_WIDTH: usize = 15;
    pub(super) const MENU_BUTTON_PADDING: usize = 10;
    pub(super) const MENU_BUTTON_HEIGHT: usize = 100;

    pub(super) const MENU_PADDING_VERTICAL: usize = 14;
    pub(super) const MENU_PADDING_HORIZONTAL: usize = 22;

    pub(super) const MENU_BAR_BUTTON_HEIGHT: usize = 50;
    pub(super) const MENU_BAR_TOTAL_HEIGHT: usize = MENU_BAR_BUTTON_HEIGHT + MENU_PADDING_VERTICAL * 2;

    pub(super) fn spawn_menu_bar(
        ui_assets: &UiAssets,
        font_builder: &FontBuilder,
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
        mut commands: Commands
    ) -> Entity {
        let menu_bar = commands.spawn((
            MenuBarRoot,
            widget::ui_background(ui_assets, texture_atlas_layouts, UiBackgroundStyle::Main),
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,

                position_type: PositionType::Absolute,
                top: px(0),
                left: px(0),
                right: px(0),
                height: px(MENU_BAR_TOTAL_HEIGHT),

                padding: UiRect::px(
                    MENU_PADDING_HORIZONTAL as f32,
                    MENU_PADDING_HORIZONTAL as f32,
                    MENU_PADDING_VERTICAL as f32,
                    MENU_PADDING_VERTICAL as f32,
                ),

                ..Default::default()
            },
        )).id();

        let menu_bar_buttons = commands.spawn((
            MenuBarButtonsRoot,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,

                width: percent(100),
                height: percent(100),

                ..Default::default()
            },
        )).id();
        commands.entity(menu_bar).add_child(menu_bar_buttons);

        let file_button = commands.spawn(widget::sized_button(
            ui_assets,
            texture_atlas_layouts,
            "File",
            px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
            percent(MENU_BUTTON_HEIGHT),
            24.,
            font_builder,
            file_button_clicked,
        )).id();
        commands.entity(menu_bar_buttons).add_child(file_button);

        let edit_button = commands.spawn(widget::sized_button(
            ui_assets,
            texture_atlas_layouts,
            "Edit",
            px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
            percent(MENU_BUTTON_HEIGHT),
            24.,
            font_builder,
            edit_button_clicked,
        )).id();
        commands.entity(menu_bar_buttons).add_child(edit_button);

        let view_button = commands.spawn(widget::sized_button(
            ui_assets,
            texture_atlas_layouts,
            "View",
            px(MENU_BUTTON_PER_CHAR_WIDTH * 4 + MENU_BUTTON_PADDING),
            percent(MENU_BUTTON_HEIGHT),
            24.,
            font_builder,
            view_button_clicked,
        )).id();
        commands.entity(menu_bar_buttons).add_child(view_button);

        let tools_button = commands.spawn(widget::sized_button(
            ui_assets,
            texture_atlas_layouts,
            "Tools",
            px(MENU_BUTTON_PER_CHAR_WIDTH * 5 + MENU_BUTTON_PADDING),
            percent(MENU_BUTTON_HEIGHT),
            24.,
            font_builder,
            tools_button_clicked,
        )).id();
        commands.entity(menu_bar_buttons).add_child(tools_button);

        let window_button = commands.spawn(widget::sized_button(
            ui_assets,
            texture_atlas_layouts,
            "Window",
            px(MENU_BUTTON_PER_CHAR_WIDTH * 6 + MENU_BUTTON_PADDING),
            percent(MENU_BUTTON_HEIGHT),
            24.,
            font_builder,
            window_button_clicked,
        )).id();
        commands.entity(menu_bar_buttons).add_child(window_button);

        menu_bar
    }

    fn file_button_clicked(
        _: On<Pointer<Click>>,
    ) {}

    fn edit_button_clicked(
        _: On<Pointer<Click>>,
    ) {}

    fn view_button_clicked(
        _: On<Pointer<Click>>,
    ) {}

    fn tools_button_clicked(
        _: On<Pointer<Click>>,
    ) {}

    fn window_button_clicked(
        _: On<Pointer<Click>>,
    ) {}
}