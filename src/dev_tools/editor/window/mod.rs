use crate::menus::font::FontBuilder;
use crate::screens::Screen;
use crate::theme::widget;
use crate::theme::widget::{UiAssets, UiBackgroundStyle};
use bevy::prelude::*;
use crate::dev_tools::editor::window::menu_bar::{spawn_menu_bar, MENU_BAR_TOTAL_HEIGHT};

mod menu_bar;

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

#[derive(Component, Debug, Clone, Default)]
struct EditorContentRoot;

#[derive(Component, Debug, Clone, Default)]
struct EditorLeftPanel;

#[derive(Component, Debug, Clone, Default)]
struct EditorCenterPanel;

#[derive(Component, Debug, Clone, Default)]
struct EditorRightPanel;

#[derive(Component, Debug, Clone, Default)]
struct EditorBottomPanel;

const CENTER_PANEL_HEIGHT: usize = 70;
const LOWER_PANEL_HEIGHT: usize = 100 - CENTER_PANEL_HEIGHT;

const LEFT_PANEL_WIDTH: usize = 25;
const RIGHT_PANEL_WIDTH: usize = 25;
const CENTER_PANEL_WIDTH: usize = 100 - LEFT_PANEL_WIDTH - RIGHT_PANEL_WIDTH;

fn spawn_editor_content(
    ui_assets: &UiAssets,
    _font_builder: &FontBuilder,
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

    fn spawn_panel<C: Component + Default>(
        width: usize,
        parent: Entity,
        commands: &mut Commands,
    ) -> Entity {
        let panel = commands
            .spawn((
                C::default(),
                Node {
                    position_type: PositionType::Relative,

                    width: percent(width),
                    height: percent(100),

                    ..Default::default()
                },
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

    let left_panel = spawn_panel::<EditorLeftPanel>(
        LEFT_PANEL_WIDTH,
        center_columns,
        &mut commands
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

    spawn_panel::<EditorCenterPanel>(
        CENTER_PANEL_WIDTH,
        center_columns,
        &mut commands
    );

    let right_panel = spawn_panel::<EditorRightPanel>(
        RIGHT_PANEL_WIDTH,
        center_columns,
        &mut commands
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

    editor_content
}