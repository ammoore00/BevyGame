use crate::dev_tools::editor::file_manager::{EditorFile, FileManager};
use crate::dev_tools::editor::window::BACKGROUND_BLEED;
use crate::screens::Screen;
use crate::theme::palette::BUTTON_TEXT;
use crate::theme::widgets;
use crate::theme::widgets::text::{text, LARGE_FONT_SIZE, SMALL_FONT_SIZE};
use crate::theme::widgets::{button, UiBackgroundStyle};
use bevy::prelude::*;
use std::collections::HashSet;
use common::marker;

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

marker!(EditorPort);
marker!(FileTabs);
marker!(EditorPortContent);

pub(super) fn spawn_editor_port() -> impl Scene {
    bsn! [
        #EditorPort
        EditorPort
        widgets::ui_root()
        Node {
            position_type: PositionType::Relative,
            justify_content: JustifyContent::FlexStart,
        }
        Children [
            file_tabs(),
            editor_port_content(),
        ]
    ]
}

fn file_tabs() -> impl Scene {
    bsn! [
        #FileTabs
        FileTabs
        widgets::ui_root()
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            position_type: PositionType::Relative,
            height: px(FILE_TABS_BUTTON_HEIGHT),
        }
        Children [
            #FileTabsBackground
            widgets::ui_background(UiBackgroundStyle::Panel)
            Node {
                position_type: PositionType::Absolute,

                left: px(0),
                right: px(0),
                top: px(-BACKGROUND_BLEED),
                bottom: px(0),
            }
            Pickable::IGNORE
        ]
    ]
}

fn editor_port_content() -> impl Scene {
    bsn! [
        #EditorPortContent
        EditorPortContent
        Node {
            width: percent(100.0),
            height: percent(100.0),

            padding: UiRect::all(px(50.0))
        }
        text("Editor Content", LARGE_FONT_SIZE, BUTTON_TEXT)
    ]
}

const FILE_TABS_BUTTON_HEIGHT: usize = 40;

const FILE_TABS_BUTTON_PER_CHAR_WIDTH: usize = 10;
const FILE_TABS_BUTTON_PADDING: usize = 10;

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
    mut commands: Commands,
) {
    let Ok((file_tabs, children)) = file_tabs_query.single() else {
        error!("Failed to get file tabs entity");
        return;
    };

    let children = children.map_or(Vec::new(), |c| c.to_vec());

    // Collect existing buttons and level them to open files
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
        if existing_buttons.contains(open_file) {
            continue;
        }

        let label = open_file.loc().id().to_string();
        let label_len = label.len();

        let file_button = commands.spawn_scene(bsn! [
                button::with_text_ext(
                    label,
                    SMALL_FONT_SIZE,
                    BUTTON_TEXT,
                    px(FILE_TABS_BUTTON_PER_CHAR_WIDTH * label_len + FILE_TABS_BUTTON_PADDING),
                    percent(100.0),
                    on_file_button_clicked
                )
            ])
            .insert(FileTabButton(open_file.clone()))
            .id();
        commands.entity(file_tabs).add_child(file_button);
    }
}

fn on_file_button_clicked(
    event: On<Pointer<Click>>,
    file_query: Query<&FileTabButton>,
    mut file_manager: ResMut<FileManager>,
) {
    let Ok(file_button) = file_query.get(event.entity) else {
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
        .map(|file| format!("Active file: {}\nFile type: {:?}", file.loc(), file.kind()))
        .unwrap_or("No active file".to_string());

    *text = Text(display);
}