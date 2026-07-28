pub mod background;
pub mod button;
pub mod text;
pub mod theme;

use bevy::prelude::*;

pub struct WidgetsPlugin;
impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((button::plugin, theme::plugin));
    }
}
