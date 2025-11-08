//! Player-specific behavior.

use bevy::prelude::*;
use std::time::Duration;

use crate::game::character::animation::{
    AnimationCapabilities, CharacterAnimation, CharacterAnimationData,
};
use crate::game::character::{CharacterState, CharacterStateEvent, Facing, character};
use crate::game::grid::coords::{
    WorldPosition, rotate_screen_space_to_facing, rotate_screen_space_to_movement,
};
//use crate::game::object::Shadow;
use crate::game::character::health::{DamageType, Health, HealthEvent, HealthEventType};
use crate::game::particle::{ParticleAnimation, ParticleSpawnEvent};
use crate::game::physics::components::{Collider, PhysicsData};
use crate::game::physics::movement::MovementController;
use crate::gamepad::GamepadRes;
use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource};
use crate::game::character::stamina::{Stamina, StaminaEvent};
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    app.add_systems(
        Update,
        (
            (
                record_action_input,
                record_player_movement_input,
                record_aim_input,
            )
                .chain()
                .run_if(in_state(Screen::Gameplay))
                .in_set(AppSystems::RecordInput)
                .in_set(PausableSystems),
            camera_follow_player.in_set(AppSystems::Respond),
        ),
    )
    .add_observer(on_aim_facing_changed)
    .add_observer(on_player_attack);
}

/// The player character.
pub fn player(
    position: Vec3,
    max_speed: f32,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    scale: f32,
) -> impl Bundle {
    let idle_layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 12, 8, None, None);
    let walk_layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 8, 8, None, None);
    let run_layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 8, 8, None, None);
    let attack_layout = TextureAtlasLayout::from_grid(UVec2::new(96, 96), 7, 8, None, None);

    let idle_layout = texture_atlas_layouts.add(idle_layout);
    let walk_layout = texture_atlas_layouts.add(walk_layout);
    let run_layout = texture_atlas_layouts.add(run_layout);
    let attack_layout = texture_atlas_layouts.add(attack_layout);

    let character_animation = CharacterAnimation::new(AnimationCapabilities {
        idle: CharacterAnimationData {
            image: player_assets.idle.clone(),
            atlas: TextureAtlas {
                layout: idle_layout,
                index: 0,
            },
            frames: 12,
            interval: Duration::from_millis(150),
        },
        walk: Some(CharacterAnimationData {
            image: player_assets.walk.clone(),
            atlas: TextureAtlas {
                layout: walk_layout,
                index: 0,
            },
            frames: 8,
            interval: Duration::from_millis(50),
        }),
        run: Some(CharacterAnimationData {
            image: player_assets.run.clone(),
            atlas: TextureAtlas {
                layout: run_layout,
                index: 0,
            },
            frames: 8,
            interval: Duration::from_millis(50),
        }),
        attack: Some(CharacterAnimationData {
            image: player_assets.attack.clone(),
            atlas: TextureAtlas {
                layout: attack_layout,
                index: 0,
            },
            frames: 7,
            interval: Duration::from_millis(ATTACK_DURATION / 7),
        }),
    });

    let sprite = character_animation.default_sprite();

    let movement_controller = MovementController {
        max_speed,
        ..default()
    };

    let character_data = character(
        "Player",
        position,
        sprite,
        character_animation,
        Collider::vertical_capsule(1.25, 0.25, position),
        scale,
    );

    //let shadow = player_assets.shadow.clone();
    //let shadow = (
    //    Shadow,
    //    Sprite {
    //        image: shadow,
    //        color: Color::srgba(1.0, 1.0, 1.0, 0.75),
    //        ..default()
    //    },
    //    Transform::from_translation(Vec3::new(0.25 * scale, -0.375 * scale, -0.1)),
    //);

    let indicator_ring_sprite = player_assets.indicator_ring.clone();
    let indicator_ring_layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 8, 1, None, None);
    let indicator_ring_layout = texture_atlas_layouts.add(indicator_ring_layout);

    let indicator_ring = (
        AimFacing::default(),
        Sprite {
            image: indicator_ring_sprite,
            texture_atlas: Some(TextureAtlas {
                layout: indicator_ring_layout,
                index: 0,
            }),
            color: Color::srgba(1.0, 1.0, 1.0, 0.25),
            ..default()
        },
        Visibility::Hidden,
        Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
    );

    (
        Player,
        movement_controller,
        character_data,
        Health::with_current(600, 400),
        Stamina::new(450, 150, 1.0),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn(indicator_ring);
            //parent.spawn(shadow);
        })),
    )
}

const COYOTE_TIME: f32 = 0.2;
const COYOTE_TIME_HEIGHT_THRESHOLD: f32 = 0.1;
const JUMP_VELOCITY: f32 = 2.75;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Eq, Reflect)]
pub struct AimFacing(pub Option<Facing>);

#[derive(EntityEvent, Debug, Clone, Reflect)]
pub struct AimFacingEvent {
    entity: Entity,
    facing: Option<Facing>,
}

fn record_aim_input(
    gamepad_res: Option<Res<GamepadRes>>,
    gamepads: Query<&Gamepad>,
    aim_query: Query<(Entity, &AimFacing)>,
    mut commands: Commands,
) {
    // Add gamepad input if available
    if let Some(gamepad_res) = gamepad_res
        && let Ok(gamepad) = gamepads.get(gamepad_res.0)
    {
        let right_stick_x = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
        let right_stick_y = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);

        // Apply deadzone
        let new_facing = if right_stick_x.abs() > 0.1 || right_stick_y.abs() > 0.1 {
            let aim_direction = Vec2::new(right_stick_x, right_stick_y);
            Some(Facing::from(rotate_screen_space_to_facing(aim_direction)))
        } else {
            None
        };

        if let Ok((aiming_entity, aim_facing)) = aim_query.single()
            && new_facing != aim_facing.0
        {
            commands.trigger(AimFacingEvent {
                entity: aiming_entity,
                facing: new_facing,
            })
        }
    }
}

fn on_aim_facing_changed(
    event: On<AimFacingEvent>,
    mut query: Query<(&mut AimFacing, &mut Sprite, &mut Visibility)>,
) {
    let Ok((mut aim_facing, mut sprite, mut visibility)) = query.get_mut(event.entity) else {
        return;
    };

    if let Some(new_facing) = event.facing {
        aim_facing.0 = Some(new_facing);
        visibility
            .set(Box::new(Visibility::Inherited))
            .expect("Failed to set visibility");
        sprite.texture_atlas.as_mut().unwrap().index = new_facing as usize;
    } else {
        aim_facing.0 = None;
        visibility
            .set(Box::new(Visibility::Hidden))
            .expect("Failed to set visibility");
    }
}

fn record_player_movement_input(
    input: Res<ButtonInput<KeyCode>>,
    gamepad_res: Option<Res<GamepadRes>>,
    gamepads: Query<&Gamepad>,
    mut controller_query: Query<
        (
            Entity,
            &mut MovementController,
            &PhysicsData,
            &WorldPosition,
            &CharacterState,
        ),
        With<Player>,
    >,
    mut commands: Commands,
) {
    let mut intent = Vec3::ZERO;

    let mut is_jumping = false;

    let mut toggle_run = false;

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

        if gamepad.just_pressed(GamepadButton::LeftThumb) {
            toggle_run = true;
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

        if input.just_pressed(KeyCode::ShiftLeft) {
            toggle_run = true;
        }

        // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
        intent = intent.normalize_or_zero();
        intent = rotate_screen_space_to_movement(intent);
    }

    // Apply movement intent to controllers.
    for (entity, mut controller, physics, position, state) in &mut controller_query {
        if !state.is_movement() {
            controller.intent = Vec3::ZERO;
            continue;
        }

        controller.intent = intent;

        let new_state = if intent.length() > 1e-6 {
            if intent.length() < 0.7 {
                controller.running = false;
                CharacterState::Walking
            } else {
                match (toggle_run, controller.running) {
                    (false, false) => CharacterState::Walking,
                    (false, true) => CharacterState::Running,
                    (true, false) => {
                        controller.running = true;
                        CharacterState::Running
                    }
                    (true, true) => {
                        controller.running = false;
                        CharacterState::Walking
                    }
                }
            }
        } else {
            controller.running = false;
            CharacterState::Idle
        };

        commands.trigger(CharacterStateEvent::new(entity, new_state));

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

fn record_action_input(
    _input: Res<ButtonInput<KeyCode>>,
    gamepad_res: Option<Res<GamepadRes>>,
    gamepads: Query<&Gamepad>,
    mut player_query: Query<(Entity, &CharacterState, &mut Facing, &Stamina), With<Player>>,
    aim_facing_query: Query<&AimFacing>,
    mut commands: Commands,
) {
    if let Some(gamepad_res) = gamepad_res
        && let Ok(gamepad) = gamepads.get(gamepad_res.0)
    {
        let Ok((player, state, mut facing, stamina)) = player_query.single_mut() else {
            return;
        };

        let Ok(aim_facing) = aim_facing_query.single() else {
            panic!("Singular aim facing not found");
        };

        if gamepad.just_pressed(GamepadButton::West) {
            commands.trigger(HealthEvent::new(
                player,
                HealthEventType::Damage(10, DamageType::Generic),
            ));
        }

        if gamepad.just_pressed(GamepadButton::North) {
            commands.trigger(HealthEvent::new(player, HealthEventType::Heal(10)));
        }

        if gamepad.just_pressed(GamepadButton::RightTrigger) && state.is_movement() && stamina.current > 0 {
            if let Some(aim_facing) = aim_facing.0 {
                *facing = aim_facing;
            }

            commands.trigger(PlayerAttackEvent {
                entity: player,
                facing: *facing,
            });
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

#[derive(EntityEvent, Debug, Clone, Reflect)]
struct PlayerAttackEvent {
    entity: Entity,
    facing: Facing,
}

const ATTACK_DURATION: u64 = 350;
const ATTACK_STAMINA_COST: usize = 50;

fn on_player_attack(
    event: On<PlayerAttackEvent>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    commands.trigger(StaminaEvent::new(event.entity, ATTACK_STAMINA_COST));

    commands.trigger(CharacterStateEvent::new(
        event.entity,
        CharacterState::Attacking {
            time_left: ATTACK_DURATION as f32 / 1000.0,
        },
    ));

    const NUM_FRAMES: u32 = 7;

    let particle_layout =
        TextureAtlasLayout::from_grid(UVec2::new(96, 96), NUM_FRAMES, 8, None, None);
    let particle_layout = texture_atlas_layouts.add(particle_layout);

    let particle_sprite = Sprite::from_atlas_image(
        player_assets.attack_particle.clone(),
        TextureAtlas {
            layout: particle_layout,
            index: event.facing as usize,
        },
    );

    let particle_animation = ParticleAnimation::new(
        event.facing as usize * NUM_FRAMES as usize,
        NUM_FRAMES as usize,
        Duration::from_millis(ATTACK_DURATION / NUM_FRAMES as u64),
    );

    commands.trigger(ParticleSpawnEvent::with_parent(
        particle_sprite,
        particle_animation,
        event.entity,
    ));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    idle: Handle<Image>,
    #[dependency]
    walk: Handle<Image>,
    #[dependency]
    run: Handle<Image>,

    #[dependency]
    attack: Handle<Image>,
    #[dependency]
    attack_particle: Handle<Image>,

    #[dependency]
    indicator_ring: Handle<Image>,

    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            idle: assets.load("images/characters/idle.png"),
            walk: assets.load("images/characters/walk.png"),
            run: assets.load("images/characters/run.png"),

            attack: assets.load("images/characters/attack.png"),
            attack_particle: assets.load("images/characters/attack_particle.png"),

            indicator_ring: assets.load("images/characters/indicator_ring.png"),

            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
