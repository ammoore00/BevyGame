use bevy::input::gamepad::{GamepadConnection, GamepadEvent};
use bevy::prelude::*;
use crate::AppSystems;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            gamepad_connections.in_set(AppSystems::RecordInput),
        )
    );
}

#[derive(Resource)]
pub struct GamepadRes(pub Entity);

pub fn gamepad_connections(
    mut commands: Commands,
    my_gamepad: Option<Res<GamepadRes>>,
    mut evr_gamepad: MessageReader<GamepadEvent>,
    gamepads: Query<Entity, With<Gamepad>>,
) {
    // If we don't have a gamepad registered yet, check if any are already connected
    if my_gamepad.is_none()
            && let Some(gamepad_entity) = gamepads.iter().next()
    {
        commands.insert_resource(GamepadRes(gamepad_entity));
    }

    for ev in evr_gamepad.read() {
        // we only care about connection events
        let GamepadEvent::Connection(ev_conn) = ev else {
            continue;
        };
        match &ev_conn.connection {
            GamepadConnection::Connected { .. } => {
                // if we don't have any gamepad yet, use this one
                if my_gamepad.is_none() {
                    commands.insert_resource(GamepadRes(ev_conn.gamepad));
                }
            }
            GamepadConnection::Disconnected => {
                // if it's the one we previously used for the player, remove it:
                if let Some(GamepadRes(old_id)) = my_gamepad.as_deref()
                        && *old_id == ev_conn.gamepad
                {
                    commands.remove_resource::<GamepadRes>();
                }
                
                //TODO: fallback to another gamepad if this one is disconnected
            }
        }
    }
}

pub fn gamepad_just_pressed(
    button: GamepadButton,
) -> impl SystemCondition<()> {
    IntoSystem::into_system(move |gamepad_res: Option<Res<GamepadRes>>, gamepad: Query<&Gamepad>| {
        if let Some(gamepad_res) = gamepad_res
                && let Ok(gamepad) = gamepad.get(gamepad_res.0)
        {
            return gamepad.just_pressed(button)
        }

        false
    })
}