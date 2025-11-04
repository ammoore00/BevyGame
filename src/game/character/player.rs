//! Player-specific behavior.

use bevy::prelude::*;

use crate::game::character::animation::CharacterAnimation;
use crate::game::character::character;
use crate::game::character::legacy_animation::PlayerAnimation;
use crate::game::grid::coords::{WorldPosition, rotate_screen_space_to_movement};
use crate::game::object::Shadow;
use crate::game::physics::components::{Collider, PhysicsData};
use crate::game::physics::movement::MovementController;
use crate::gamepad::GamepadRes;
use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            record_player_movement_input
                .in_set(AppSystems::RecordInput)
                .in_set(PausableSystems),
            camera_follow_player.in_set(AppSystems::Update),
        ),
    );
}

/// The player character.
pub fn player(
    position: Vec3,
    max_speed: f32,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    scale: f32,
) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(15), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();
    let sprite = Sprite::from_atlas_image(
        player_assets.ducky.clone(),
        TextureAtlas {
            layout: texture_atlas_layout,
            index: player_animation.get_atlas_index(),
        },
    );

    let character_data = character(
        "Player",
        position,
        sprite,
        CharacterAnimation::Dynamic,
        Collider::vertical_capsule(0.75, 0.25, position),
        scale,
    );

    let shadow = player_assets.shadow.clone();
    let shadow = (
        Shadow,
        Sprite {
            image: shadow,
            color: Color::srgba(1.0, 1.0, 1.0, 0.75),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.25 * scale, -0.375 * scale, -0.1)),
    );

    (
        Player,
        MovementController {
            max_speed,
            ..default()
        },
        PlayerAnimation::new(),
        character_data,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn(shadow);
        })),
    )
}

const COYOTE_TIME: f32 = 0.2;
const COYOTE_TIME_HEIGHT_THRESHOLD: f32 = 0.1;
const JUMP_VELOCITY: f32 = 2.75;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

fn record_player_movement_input(
    input: Res<ButtonInput<KeyCode>>,
    gamepad_res: Option<Res<GamepadRes>>,
    gamepads: Query<&Gamepad>,
    mut controller_query: Query<
        (&mut MovementController, &PhysicsData, &WorldPosition),
        With<Player>,
    >,
) {
    let mut intent = Vec3::ZERO;
    let mut is_jumping = false;

    // Add gamepad input if available
    if let Some(gamepad_res) = gamepad_res
        && let Ok(gamepad) = gamepads.get(gamepad_res.0)
    {
        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);

        // Apply deadzone
        if left_stick_x.abs() > 0.1 || left_stick_y.abs() > 0.1 {
            intent.x += left_stick_x;
            intent.z -= left_stick_y;

            intent = rotate_screen_space_to_movement(intent);
        }

        if gamepad.just_pressed(GamepadButton::South) {
            is_jumping = true;
        }
    }

    if intent == Vec3::ZERO {
        // Collect directional input.
        if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
            intent.z -= 1.0;
        }
        if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
            intent.z += 1.0;
        }
        if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
            intent.x -= 1.0;
        }
        if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
            intent.x += 1.0;
        }

        if input.just_pressed(KeyCode::Space) {
            is_jumping = true;
        }

        // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
        intent = intent.normalize_or_zero();
        intent = rotate_screen_space_to_movement(intent);
    }

    // Apply movement intent to controllers.
    for (mut controller, physics, position) in &mut controller_query {
        controller.intent = intent;
        if let PhysicsData::Kinematic {
            time_since_grounded,
            last_grounded_height,
            ..
        } = *physics
            && time_since_grounded < COYOTE_TIME
            && position.as_vec3().y < last_grounded_height + COYOTE_TIME_HEIGHT_THRESHOLD
            && is_jumping
        {
            controller.intent.y = JUMP_VELOCITY;
        }
    }
}

fn camera_follow_player(
    player_query: Query<&mut Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    // Update camera position to match player position
    camera_transform.translation = player_transform.translation;
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    ducky: Handle<Image>,
    #[dependency]
    shadow: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load("images/ducky2.png"),
            shadow: assets.load("images/ducky_shadow.png"),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
