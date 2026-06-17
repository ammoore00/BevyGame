use crate::menus::font::FontBuilder;
use crate::screens::Screen;
use crate::theme::widget;
use crate::theme::widget::{UiAssets, UiBackgroundStyle};
use bevy::prelude::*;
use crate::dev_tools::editor::window::browser::spawn_file_browser;
use crate::dev_tools::editor::window::editor_port::spawn_editor_port;
use crate::dev_tools::editor::window::menu_bar::{spawn_menu_bar, MENU_BAR_TOTAL_HEIGHT};

mod menu_bar;
mod browser;
mod editor_port;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        browser::plugin,
        editor_port::plugin,
    ));
    
    app.add_systems(OnEnter(Screen::Editor), spawn_editor_window);
}

pub(super) const MENU_BUTTON_PER_CHAR_WIDTH: usize = 15;
pub(super) const MENU_BUTTON_PADDING: usize = 10;
pub(super) const MENU_BUTTON_HEIGHT: usize = 48;

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

#[derive(Component, Debug, Clone, Default, Copy)]
struct EditorContentRoot;

#[derive(Component, Debug, Clone, Default, Copy)]
struct EditorLeftPanel;

#[derive(Component, Debug, Clone, Default, Copy)]
struct EditorCenterPanel;

#[derive(Component, Debug, Clone, Default, Copy)]
struct EditorRightPanel;

#[derive(Component, Debug, Clone, Default, Copy)]
struct EditorBottomPanel;

const CENTER_PANEL_HEIGHT: usize = 70;
const LOWER_PANEL_HEIGHT: usize = 100 - CENTER_PANEL_HEIGHT;

const LEFT_PANEL_WIDTH_TARGET: usize = 25;
const RIGHT_PANEL_WIDTH_TARGET: usize = 25;
const CENTER_PANEL_WIDTH: usize = 100 - LEFT_PANEL_WIDTH_TARGET - RIGHT_PANEL_WIDTH_TARGET;

const LEFT_PANEL_MAX_WIDTH: usize = 500;
const RIGHT_PANEL_MAX_WIDTH: usize = 800;

const PANEL_PADDING: usize = 14;

const BACKGROUND_BLEED: f32 = 10.0;

fn spawn_editor_content(
    ui_assets: &UiAssets,
    font_builder: &FontBuilder,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    mut commands: Commands,
) -> Entity {
    
    //------ Spawn layout ------//
    
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

    let left_panel = spawn_panel::<EditorLeftPanel>(
        center_columns,
        &mut commands,
        Some(LEFT_PANEL_WIDTH_TARGET),
        Some(LEFT_PANEL_MAX_WIDTH),
        Some(AlignSelf::FlexStart),
        Some(UiRect::all(px(PANEL_PADDING))),
    );
    spawn_panel_background(
        left_panel,
        ui_assets,
        texture_atlas_layouts,
        UiBackgroundStyle::Panel,
        BACKGROUND_BLEED,
        BACKGROUND_BLEED,
        &mut commands,
    );

    let center_panel = spawn_panel::<EditorCenterPanel>(
        center_columns,
        &mut commands,
        None,
        None,
        None,
        None,
    );

    let right_panel = spawn_panel::<EditorRightPanel>(
        center_columns,
        &mut commands,
        Some(RIGHT_PANEL_WIDTH_TARGET),
        Some(RIGHT_PANEL_MAX_WIDTH),
        Some(AlignSelf::FlexEnd),
        Some(UiRect::all(px(PANEL_PADDING))),
    );
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
    
    //------ Spawn Content ------//
    
    let file_browser = spawn_file_browser(
        ui_assets,
        font_builder,
        texture_atlas_layouts,
        commands.reborrow()
    );
    commands.entity(left_panel).add_child(file_browser);

    let editor_port = spawn_editor_port(
        ui_assets,
        texture_atlas_layouts,
        commands.reborrow()
    );
    commands.entity(center_panel).add_child(editor_port);

    editor_content
}

fn spawn_panel<C: Component + Default>(
    parent: Entity,
    commands: &mut Commands,
    width: Option<usize>,
    max_width: Option<usize>,
    align_self: Option<AlignSelf>,
    padding: Option<UiRect>,
) -> Entity {
    let mut node = Node {
        position_type: PositionType::Relative,

        height: percent(100),

        ..Default::default()
    };

    if let Some(width) = width {
        node.width = percent(width);
    }

    if let Some(max_width) = max_width {
        node.max_width = px(max_width);
    } else {
        node.flex_grow = 1.0;
    }

    if let Some(align_self) = align_self {
        node.align_self = align_self;
    }

    if let Some(padding) = padding {
        node.padding = padding;
    }

    let panel = commands
        .spawn((
            C::default(),
            node,
            Pickable::IGNORE,
        ))
        .id();

    commands.entity(parent).add_child(panel);
    panel
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