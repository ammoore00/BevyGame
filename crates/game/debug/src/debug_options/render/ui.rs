use bevy::prelude::*;
use common::dev_tools::DebugState;
use crate::debug_options::options::UiRenderRes;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, set_ui_render.run_if(resource_changed::<UiRenderRes>));
}

fn set_ui_render(
    ui_debug_setting: Res<UiRenderRes>,
    mut internal_setting: ResMut<GlobalUiDebugOptions>,
) {
    *internal_setting = GlobalUiDebugOptions {
        enabled: ui_debug_setting.get(),
        ..*internal_setting
    };
}