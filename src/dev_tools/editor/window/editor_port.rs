use crate::dev_tools::editor::file_manager::{EditorFile, FileManager};
use crate::dev_tools::editor::window::BACKGROUND_BLEED;
use crate::menus::font::FontBuilder;
use crate::screens::Screen;
use crate::theme::palette::BUTTON_TEXT;
use crate::theme::widget;
use crate::theme::widget::{UiAssets, UiBackgroundStyle, LARGE_FONT_SIZE, SMALL_FONT_SIZE};
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use std::collections::HashSet;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_file_tab_buttons,
            update_editor_port_content,
        )
            .run_if(in_state(Screen::Editor))
    );
}

#[derive(Component, Debug, Clone, Default, Copy)]
struct EditorPort;

pub(super) fn spawn_editor_port(
    ui_assets: &UiAssets,
    font_builder: &FontBuilder,
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

    let editor_port_content = commands.spawn(editor_port_content(
        ui_assets,
        font_builder,
        texture_atlas_layouts
    )).id();
    commands.entity(editor_port).add_child(editor_port_content);

    editor_port
}

const FILE_TABS_BUTTON_HEIGHT: usize = 40;

const FILE_TABS_BUTTON_PER_CHAR_WIDTH: usize = 10;
const FILE_TABS_BUTTON_PADDING: usize = 10;

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
            width: percent(100.0),
            height: px(FILE_TABS_BUTTON_HEIGHT),

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
            && !file_manager.is_file_open(&button.0)
        {
            commands.entity(button_entity).despawn();
        }
    }

    // Spawn buttons for new open files
    for open_file in file_manager.open_files() {
        let open_file = &*open_file.read().unwrap();
        
        if existing_buttons.contains(open_file) {
            continue;
        }

        let label = open_file.loc().id().to_string();
        let label_len = label.len();

        let file_button = commands.spawn((
            FileTabButton(open_file.clone()),
            widget::sized_button(
                &ui_assets,
                &mut texture_atlas_layouts,
                label,
                px(FILE_TABS_BUTTON_PER_CHAR_WIDTH * label_len + FILE_TABS_BUTTON_PADDING),
                percent(100.0),
                SMALL_FONT_SIZE,
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

    match event.button {
        PointerButton::Primary => {
            file_manager.set_active_file(&file_button.0).unwrap_or_else(|err| {
                error!("Failed to set active file: {:?}", err);
            });

            info!("Set active file to: {}", file_button.0.loc());
        }
        PointerButton::Secondary => {},
        PointerButton::Middle => file_manager.close(&file_button.0),
    }
}

#[derive(Component, Debug, Clone, Default, Copy)]
struct EditorPortContent;

pub(super) fn editor_port_content(
    _ui_assets: &UiAssets,
    font_builder: &FontBuilder,
    _texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    (
        EditorPortContent,
        Node {
            width: percent(100.0),
            height: percent(100.0),

            padding: UiRect::all(px(50.0)),

            ..Default::default()
        },
        widget::text("Editor Content", font_builder, LARGE_FONT_SIZE, BUTTON_TEXT)
    )
}

fn update_editor_port_content(
    mut content_query: Query<
        &mut Text,
        With<EditorPortContent>
    >,
    file_manager: Res<FileManager>,
) {
    let Ok(mut text) = content_query.single_mut() else {
        error!("Failed to get editor port content");
        return;
    };

    let display = file_manager.active_file()
        .map(|file| {
            let file = &*file.read().unwrap();
            format!("Active file: {}\nFile type: {:?}", file.loc(), file.kind())
        })
        .unwrap_or("No active file".to_string());

    *text = Text(display);
}