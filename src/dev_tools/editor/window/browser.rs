use bevy::prelude::*;
use crate::theme::widget::{styled_button, ButtonStyle, UiAssets};

#[derive(Component, Debug, Clone, Default, Copy)]
struct FileBrowser;

pub(super) fn spawn_file_browser(
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    mut commands: Commands,
) -> Entity {
    let browser = commands.spawn((
        FileBrowser,
        Node {
            width: percent(100),
            height: percent(100),

            ..Default::default()
        },
    )).id();

    let button = commands.spawn((
        styled_button(
            ui_assets,
            texture_atlas_layouts,
            2,
            |_: On<Pointer<Click>>,| {},
            ButtonStyle::ArrowRight,
        ),
    )).id();
    commands.entity(browser).add_child(button);

    browser
}