use crate::gamepad::GamepadRes;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

pub mod gamepad;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((gamepad::plugin,));
    }
}

#[derive(SystemParam)]
pub struct InputReader<'w, 's> {
    gamepad_query: Query<'w, 's, &'static Gamepad>,
    gamepad_res: Option<Res<'w, GamepadRes>>,
    keyboard_input: Res<'w, ButtonInput<KeyCode>>,
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
}
impl<'w, 's> InputReader<'w, 's> {
    pub fn gamepad(&self) -> Option<&Gamepad> {
        self.gamepad_res
            .as_ref()
            .map(|r| r.0)
            .map(|id| self.gamepad_query.get(id).ok())
            .flatten()
    }

    pub fn keyboard(&self) -> &ButtonInput<KeyCode> {
        &self.keyboard_input
    }

    pub fn mouse_buttons(&self) -> &ButtonInput<MouseButton> {
        &self.mouse_buttons
    }
}