use crate::gamepad::{GamepadRes, GamepadStick, get_stick_with_deadzone};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::AppSystems;
use getset::{Getters, MutGetters};

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

#[derive(SystemParam, Getters, MutGetters)]
pub struct InputReader<'w, 's> {
    gamepad_query: Query<'w, 's, &'static Gamepad>,
    gamepad_res: Option<Res<'w, GamepadRes>>,
    #[getset(get = "pub")]
    keyboard: Res<'w, ButtonInput<KeyCode>>,
    #[getset(get = "pub")]
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    #[getset(get = "pub", get_mut = "pub")]
    mouse_cursor: MessageReader<'w, 's, CursorMoved>,

    #[getset(get = "pub")]
    last_input_mode: Res<'w, LastInputMode>,
}
impl<'w, 's> InputReader<'w, 's> {
    pub fn gamepad(&self) -> Option<&Gamepad> {
        self.gamepad_res
            .as_ref()
            .map(|r| r.0)
            .and_then(|id| self.gamepad_query.get(id).ok())
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
            || get_stick_with_deadzone(gamepad, GamepadStick::Right).length() > 0.
            || get_stick_with_deadzone(gamepad, GamepadStick::Left).length() > 0.
    }

    fn mouse_or_keyboard_used_this_frame(&mut self) -> bool {
        self.keyboard().get_pressed().len() > 0
            || self.mouse_buttons().get_pressed().len() > 0
            || self.mouse_cursor.read().len() > 0
    }
}

fn set_last_input_mode(
    // Input reader does not have mutable access to last input mode
    // so we need separate access to it
    mut input_reader: ParamSet<(InputReader, ResMut<LastInputMode>)>,
) {
    if input_reader.p0().gamepad_used_this_frame() {
        *input_reader.p1() = LastInputMode::Gamepad;
    } else if input_reader.p0().mouse_or_keyboard_used_this_frame() {
        *input_reader.p1() = LastInputMode::MouseAndKeyboard;
    }
}
