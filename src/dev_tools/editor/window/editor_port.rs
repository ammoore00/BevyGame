use std::collections::HashSet;
use bevy::ecs::relationship::Relationship;
use crate::dev_tools::editor::window::{BACKGROUND_BLEED, MENU_BUTTON_PADDING, MENU_BUTTON_PER_CHAR_WIDTH};
use crate::menus::font::FontBuilder;
use crate::theme::widget;
use crate::theme::widget::{UiAssets, UiBackgroundStyle, MEDIUM_FONT_SIZE};
use bevy::prelude::*;
use crate::dev_tools::editor::file_manager::{EditorFile, FileManager};
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_file_tab_buttons,
        )
            .run_if(in_state(Screen::Editor))
    );
}

#[derive(Component, Debug, Clone, Default, Copy)]
struct EditorPort;

pub(super) fn spawn_editor_port(
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    mut commands: Commands,
) -> Entity {
    let editor_port = commands.spawn((
        EditorPort,
        Node {
            flex_direction: FlexDirection::Column,

            width: Val::Percent(100.0),
            height: Val::Percent(100.0),

            ..Default::default()
        }
    )).id();

    let file_tabs = commands.spawn(file_tabs(
        ui_assets,
        texture_atlas_layouts
    )).id();
    commands.entity(editor_port).add_child(file_tabs);

    editor_port
}

pub(super) const FILE_TABS_BUTTON_HEIGHT: f32 = 48.;

#[derive(Component, Debug, Clone, Default, Copy)]
struct FileTabs;

pub(super) fn file_tabs(
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    (
        FileTabs,
        Node {
            position_type: PositionType::Relative,
            width: Val::Percent(100.0),
            height: Val::Px(FILE_TABS_BUTTON_HEIGHT),

            ..Default::default()
        },
        Pickable::IGNORE,
        children![
            // TODO: Figure out why this background isn't rendering
            widget::ui_background(ui_assets, texture_atlas_layouts, UiBackgroundStyle::Panel),
            Node {
                position_type: PositionType::Absolute,

                left: px(0),
                right: px(0),
                top: px(-BACKGROUND_BLEED),
                bottom: px(0),

                ..Default::default()
            },
            Pickable::IGNORE,
        ]
    )
}

#[derive(Component, Debug, Clone)]
struct FileTabButton(EditorFile);

fn update_file_tab_buttons(
    file_tabs_query: Query<
        (
            Entity,
            Option<&Children>
        ),
        With<FileTabs>
    >,
    file_button_query: Query<(
        Entity,
        &FileTabButton
    )>,
    file_manager: Res<FileManager>,
    ui_assets: Res<UiAssets>,
    font_builder: FontBuilder,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let Ok((file_tabs, children)) = file_tabs_query.single() else {
        error!("Failed to get file tabs entity");
        return;
    };

    let children = children.map_or(Vec::new(), |c| c.to_vec());

    // Collect existing buttons and map them to open files
    // TODO: Evaluate the performance impact of this approach
    let mut existing_buttons: HashSet<_> = HashSet::new();
    for child in children.iter() {
        if let Ok((_, button)) = file_button_query.get(*child) {
            existing_buttons.insert(button.0.clone());
        }
    }

    // Despawn buttons for files no longer open
    for child in children.iter() {
        if let Some((button_entity, button)) = file_button_query.get(*child).ok()
            && !file_manager.open_files().contains(&button.0)
        {
            commands.entity(button_entity).despawn();
        }
    }

    // Spawn buttons for new open files
    for open_file in file_manager.open_files() {
        if existing_buttons.contains(open_file) {
            continue;
        }

        let name = open_file.name();
        let name_len = name.len();

        let file_button = commands.spawn((
            FileTabButton(open_file.clone()),
            widget::sized_button(
                &ui_assets,
                &mut texture_atlas_layouts,
                name,
                px(MENU_BUTTON_PER_CHAR_WIDTH * name_len + MENU_BUTTON_PADDING),
                percent(100.0),
                MEDIUM_FONT_SIZE,
                &font_builder,
                on_file_button_clicked
            ),
        )).id();
        commands.entity(file_tabs).add_child(file_button);
    }
}

fn on_file_button_clicked(
    event: On<Pointer<Click>>,
    parent_query: Query<&ChildOf>,
    file_query: Query<&FileTabButton>,
    mut file_manager: ResMut<FileManager>,
) {
    let Ok(button_root) = parent_query.get(event.entity).map(ChildOf::get) else {
        error!("Failed to get button root");
        return;
    };

    let Ok(file_button) = file_query.get(button_root) else {
        error!("Failed to get file button");
        return;
    };

    file_manager.set_active_file(&file_button.0).unwrap_or_else(|err| {
        error!("Failed to set active file: {:?}", err);
    });

    info!("Set active file to: {}", file_button.0.loc());
}