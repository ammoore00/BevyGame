use crate::gamepad::GamepadRes;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::AppSystems;

pub mod gamepad;
pub mod mouse;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((gamepad::plugin,));

        app.init_resource::<LastInputMode>();
        app.add_systems(Update, set_last_input_mode.in_set(AppSystems::RecordInput));
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LastInputMode {
    #[default]
    Gamepad,
    MouseAndKeyboard,
}

#[derive(SystemParam)]
pub struct InputReader<'w, 's> {
    gamepad_query: Query<'w, 's, &'static Gamepad>,
    gamepad_res: Option<Res<'w, GamepadRes>>,
    keyboard_input: Res<'w, ButtonInput<KeyCode>>,
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    mouse_cursor: MessageReader<'w, 's, CursorMoved>,
}
impl<'w, 's> InputReader<'w, 's> {
    pub fn gamepad(&self) -> Option<&Gamepad> {
        self.gamepad_res
            .as_ref()
            .map(|r| r.0)
            .and_then(|id| self.gamepad_query.get(id).ok())
    }

    pub fn keyboard(&self) -> &ButtonInput<KeyCode> {
        &self.keyboard_input
    }

    pub fn mouse_buttons(&self) -> &ButtonInput<MouseButton> {
        &self.mouse_buttons
    }

    pub fn cursor_mut(&mut self) -> &mut MessageReader<'w, 's, CursorMoved> {
        &mut self.mouse_cursor
    }
}

trait LastInputReader {
    fn gamepad_used_this_frame(&self) -> bool;
    fn mouse_or_keyboard_used_this_frame(&mut self) -> bool;
}
impl LastInputReader for InputReader<'_, '_> {
    fn gamepad_used_this_frame(&self) -> bool {
        let Some(gamepad) = self.gamepad() else {
            return false;
        };

        gamepad.any_pressed(GamepadButton::all())
    }

    fn mouse_or_keyboard_used_this_frame(&mut self) -> bool {
        self.keyboard().get_pressed().len() > 0
            || self.mouse_buttons().get_pressed().len() > 0
            || self.mouse_cursor.read().len() > 0
    }
}

fn set_last_input_mode(
    mut input_reader: InputReader,
    mut last_input_mode: ResMut<LastInputMode>,
) {
    if input_reader.gamepad_used_this_frame() {
        *last_input_mode = LastInputMode::Gamepad;
    } else if input_reader.mouse_or_keyboard_used_this_frame() {
        *last_input_mode = LastInputMode::MouseAndKeyboard;
    }
}