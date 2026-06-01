use crate::screens::Screen;
use crate::theme::widget;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Editor), spawn_editor_window);
}

#[derive(Component, Debug, Clone)]
struct EditorUiRoot;

fn spawn_editor_window(mut commands: Commands) {
    commands
        .spawn((
            EditorUiRoot,
            widget::ui_root("Editor"),
            GlobalZIndex(1),
            DespawnOnExit(Screen::Editor),
        ))
        .with_children(|_parent| {});
}
