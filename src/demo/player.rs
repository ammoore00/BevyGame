//! Player-specific behavior.

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    demo::{
        animation::PlayerAnimation,
        movement::{MovementController, ScreenWrap},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            record_player_directional_input
                .in_set(AppSystems::RecordInput)
                .in_set(PausableSystems),
            gamepad::gamepad_connections
                .in_set(AppSystems::RecordInput),
        )
    );
}

/// The player character.
pub fn player(
    max_speed: f32,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    (
        Name::new("Player"),
        Player,
        Sprite::from_atlas_image(
            player_assets.ducky.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            },
        ),
        Transform::from_scale(Vec2::splat(8.0).extend(1.0)),
        MovementController {
            max_speed,
            ..default()
        },
        ScreenWrap,
        player_animation,
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Player;

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    my_gamepad: Option<Res<gamepad::MyGamepad>>,
    gamepads: Query<&Gamepad>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Add gamepad input if available
    if let Some(gamepad_res) = my_gamepad {
        if let Ok(gamepad) = gamepads.get(gamepad_res.0) {
            let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
            let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);

            debug!("Left stick x: {}, y: {}", left_stick_x, left_stick_y);

            // Apply deadzone
            if left_stick_x.abs() > 0.1 || left_stick_y.abs() > 0.1 {
                intent.x += left_stick_x;
                intent.y += left_stick_y;
            }
        }
    }

    // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
    // This should be omitted if the input comes from an analog stick instead.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.intent = intent;
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings(
                "images/ducky.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}

mod gamepad {
    use bevy::input::gamepad::{GamepadConnection, GamepadEvent};
    use bevy::prelude::*;

    #[derive(Resource)]
    pub(super) struct MyGamepad(pub(super) Entity); // Make the Entity public

    pub(super) fn gamepad_connections(
        mut commands: Commands,
        my_gamepad: Option<Res<MyGamepad>>,
        mut evr_gamepad: MessageReader<GamepadEvent>,
        gamepads: Query<Entity, With<Gamepad>>,
    ) {
        // If we don't have a gamepad registered yet, check if any are already connected
        if my_gamepad.is_none() {
            if let Some(gamepad_entity) = gamepads.iter().next() {
                debug!("Using already-connected gamepad: {:?}", gamepad_entity);
                commands.insert_resource(MyGamepad(gamepad_entity));
            }
        }

        for ev in evr_gamepad.read() {
            // we only care about connection events
            let GamepadEvent::Connection(ev_conn) = ev else {
                continue;
            };
            match &ev_conn.connection {
                GamepadConnection::Connected {
                    name, ..
                } => {
                    debug!(
                        "New gamepad connected: {:?}, name: {}",
                        ev_conn.gamepad, name,
                    );
                    // if we don't have any gamepad yet, use this one
                    if my_gamepad.is_none() {
                        commands.insert_resource(MyGamepad(ev_conn.gamepad));
                    }
                }
                GamepadConnection::Disconnected => {
                    debug!("Lost connection with gamepad: {:?}", ev_conn.gamepad);
                    // if it's the one we previously used for the player, remove it:
                    if let Some(MyGamepad(old_id)) = my_gamepad.as_deref() {
                        if *old_id == ev_conn.gamepad {
                            commands.remove_resource::<MyGamepad>();
                        }
                    }
                }
            }
        }
    }
}