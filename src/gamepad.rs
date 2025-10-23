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
pub struct MyGamepad(pub Entity);

pub fn gamepad_connections(
    mut commands: Commands,
    my_gamepad: Option<Res<MyGamepad>>,
    mut evr_gamepad: MessageReader<GamepadEvent>,
    gamepads: Query<Entity, With<Gamepad>>,
) {
    // If we don't have a gamepad registered yet, check if any are already connected
    if my_gamepad.is_none() {
        if let Some(gamepad_entity) = gamepads.iter().next() {
            commands.insert_resource(MyGamepad(gamepad_entity));
        }
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
                    commands.insert_resource(MyGamepad(ev_conn.gamepad));
                }
            }
            GamepadConnection::Disconnected => {
                // if it's the one we previously used for the player, remove it:
                if let Some(MyGamepad(old_id)) = my_gamepad.as_deref() {
                    if *old_id == ev_conn.gamepad {
                        commands.remove_resource::<MyGamepad>();
                    }
                }
                
                //TODO: fallback to another gamepad if this one is disconnected
            }
        }
    }
}