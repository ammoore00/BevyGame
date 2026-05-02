//! Player-specific behavior.

use crate::game::character::animation::{
    CharacterAnimationTracker,
};
use crate::game::character::{character, state, Character, CharacterBuilderContext, Facing};
use crate::game::level::grid::coords::{
    rotate_screen_space_to_facing, rotate_screen_space_to_movement, WorldPosition,
};
use bevy::prelude::*;
use std::any::TypeId;
use std::str::FromStr;
use std::time::Duration;
use tracing::warn;
//use crate::game::object::Shadow;
use crate::game::character::state::action_states::{Attacking, Idle, Running, Sprinting, Walking};
use crate::game::character::health::Health;
use crate::game::character::stamina::{Stamina, StaminaEvent};
use crate::game::character::state::state_transitions::ActionStateCapabilities;
use crate::game::particle::{ParticleAnimation, ParticleSpawnEvent};
use crate::game::physics::components::{Collider, PhysicsData};
use crate::game::physics::movement::MovementController;
use crate::gamepad::GamepadRes;
use crate::screens::Screen;
use crate::{asset_tracking::LoadResource, AppSystems, PausableSystems};
use crate::data::ResourceLocation;
use crate::datagen_api::assets::CharacterSpriteResource;
use crate::datagen_api::attack::{AttackContext, AttackResource};
use crate::game::character::assets::CharacterResource;
use crate::game::character::state::{is_in_movement_state, ActionState, CharacterStateEvent, ActionStateTracker};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    app.add_systems(
        Update,
        (
            // Normal Systems
            (record_aim_input,)
                .run_if(in_state(Screen::Gameplay))
                .in_set(AppSystems::RecordInput)
                .in_set(PausableSystems),
            camera_follow_player.in_set(AppSystems::Respond),
            // Exclusive Systems
            // Note: Chaining exclusive systems works if they all have the &mut World signature
            (record_action_input, record_player_movement_input)
                .chain()
                .run_if(in_state(Screen::Gameplay))
                .in_set(AppSystems::RecordInput)
                .in_set(PausableSystems),
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
    scale: f32,
    context: &CharacterBuilderContext,
) -> impl Bundle {
    let player_data_location = ResourceLocation::<CharacterResource>::from_str("player").unwrap();
    let player_data = context.character_registry().get_asset(&player_data_location)
        .expect("Failed to find player character data");

    let player_animations = player_data.resolve_animation_handles(context.animation_registry().resolved_registry());
    let idle_animation = player_animations.get(&TypeId::of::<Idle>()).cloned()
        .expect("Failed to find idle animation for player character");

    let animation_assets = context.animation_registry().resolved_assets();
    let animation_tracker =
        CharacterAnimationTracker::new(idle_animation, animation_assets);

    let sprite = animation_tracker.default_sprite(animation_assets);

    let movement_controller = MovementController {
        max_speed,
        ..default()
    };

    let character_data = character(
        player_data_location,
        position,
        sprite,
        animation_tracker,
        Collider::vertical_capsule(1.25, 0.25, position),
        scale,
        context,
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

    let indicator_ring_loc = "player/indicator_ring".parse::<ResourceLocation<CharacterSpriteResource>>().unwrap();
    let indicator_ring_sprite = context.sprite_registry.get_handle(&indicator_ring_loc).unwrap();
    let indicator_ring_layout = player_assets.indicator_ring_layout.clone();

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
        Health::new(300),
        Stamina::new(200, 200, 1.0),
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

fn record_player_movement_input(world: &mut World) {
    let mut intent = Vec3::ZERO;
    let mut is_jumping = false;
    let mut toggle_sprint = false;

    let input = world.resource::<ButtonInput<KeyCode>>();
    let gamepad_res = world.get_resource::<GamepadRes>();

    if let Some(gamepad_id) = gamepad_res.map(|r| r.0)
        && let Ok(Some(gamepad)) = world.get_entity(gamepad_id).map(|e| e.get::<Gamepad>())
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
            toggle_sprint = true;
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
            toggle_sprint = true;
        }

        // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
        intent = intent.normalize_or_zero();
        intent = rotate_screen_space_to_movement(intent);
    }

    let mut controller_query = world.query_filtered::<Entity, (
        With<Player>,
        With<Character>,
        With<MovementController>,
        With<PhysicsData>,
        With<WorldPosition>,
        With<ActionStateTracker>,
        With<ActionStateCapabilities>,
    )>();

    let entities: Vec<Entity> = controller_query.iter(world).collect();

    for entity in entities {
        // Get the current state
        let tracker = world.get::<ActionStateTracker>(entity).cloned().unwrap();
        let Some(prev_state) = state::get_state(entity, &tracker, world) else {
            warn!("Failed to get reflect component for entity {}", entity);
            continue;
        };

        // Check if the current state is movement
        let is_movement = is_in_movement_state(
            entity,
            &world.get::<ActionStateTracker>(entity).unwrap().clone(),
            world,
        );

        // Determine new state from movement intent
        let mut sprinting = {
            let controller = world.get::<MovementController>(entity).unwrap();
            controller.sprinting
        };

        let new_state: Box<dyn ActionState> = if intent.length() > 1e-6 {
            if intent.length() < 0.7 {
                sprinting = false;
                Box::new(Walking)
            } else {
                match (toggle_sprint, sprinting) {
                    // We aren't sprinting, and don't want to sprint
                    (false, false) => {
                        sprinting = false;
                        Box::new(Running)
                    }
                    // We aren't sprinting, and want to start sprinting
                    (false, true) => {
                        sprinting = true;
                        Box::new(Sprinting)
                    }
                    // We are sprinting, and want to keep sprinting
                    (true, false) => {
                        sprinting = true;
                        Box::new(Sprinting)
                    }
                    // We are sprinting, and want to stop sprinting
                    (true, true) => {
                        sprinting = false;
                        Box::new(Running)
                    }
                }
            }
        } else {
            sprinting = false;
            Box::new(Idle)
        };

        let state_capabilities = world.get::<ActionStateCapabilities>(entity).cloned().unwrap();

        // If the character state has changed
        if (*prev_state).type_id() != (*new_state).type_id() {
            // Attempt to create a state transition event
            let should_sprint = (*new_state).type_id() == TypeId::of::<Sprinting>();
            if let Ok(event) =
                CharacterStateEvent::try_new(entity, &state_capabilities, new_state, prev_state)
            {
                world.trigger(event);
                sprinting = should_sprint;
            }
        }

        // Update the controller's intent
        if let Some(mut controller) = world.get_mut::<MovementController>(entity) {
            controller.sprinting = sprinting;
            if is_movement {
                controller.intent = intent;
            } else {
                controller.intent = Vec3::ZERO;
            }
        }

        // Handle jumping
        let physics = world.get::<PhysicsData>(entity).unwrap();
        let position = world.get::<WorldPosition>(entity).unwrap();

        if let PhysicsData::Kinematic {
            time_since_grounded,
            last_grounded_height,
            ..
        } = *physics
            && time_since_grounded < COYOTE_TIME
            && position.as_vec3().y < last_grounded_height + COYOTE_TIME_HEIGHT_THRESHOLD
            && is_jumping
            && let Some(mut controller) = world.get_mut::<MovementController>(entity)
        {
            controller.intent.y = JUMP_VELOCITY;
        }
    }
}

fn record_action_input(world: &mut World) {
    let gamepad = world.get_resource::<GamepadRes>().map(|r| r.0);

    let player = {
        let mut query = world.query_filtered::<Entity, With<Player>>();
        query.single(world).ok()
    };

    let (_player, gamepad_id) = match (player, gamepad) {
        (Some(p), Some(g)) => (p, g),
        _ => return,
    };

    let gamepad = world
        .get_entity(gamepad_id)
        .unwrap()
        .get::<Gamepad>()
        .unwrap();
    let attack = gamepad.just_pressed(GamepadButton::RightTrigger);

    let mut player_query = world.query_filtered::<Entity, (
        With<Player>,
        With<Character>,
        With<Facing>,
        With<Stamina>,
        With<ActionStateTracker>,
        With<ActionStateCapabilities>,
    )>();
    let player = player_query.single(world).unwrap();

    let state_capabilities = world.get::<ActionStateCapabilities>(player).cloned().unwrap();

    // 2. Check if it's a movement state (this takes &mut World)
    let is_movement = is_in_movement_state(
        player,
        &world.get::<ActionStateTracker>(player).unwrap().clone(),
        world,
    );

    let is_idle = world
        .query_filtered::<Entity, With<Idle>>()
        .get(world, player)
        .is_ok();

    let prev_state = {
        let state_tracker = world.get::<ActionStateTracker>(player).cloned().unwrap();

        let Some(prev_state) = state::get_state(player, &state_tracker, world) else {
            error!("Failed to get reflect component for entity {}", player);
            return;
        };
        prev_state
    };

    let stamina = world.get::<Stamina>(player).cloned().unwrap();

    let mut aim_facing_query = world.query_filtered::<Entity, With<AimFacing>>();
    let aim_facing = aim_facing_query.single(world).unwrap();
    let aim_facing = world.get::<AimFacing>(aim_facing).cloned().unwrap();

    if attack && (is_movement || is_idle) && stamina.current > 0 {
        let facing = {
            let mut facing = world.get_mut::<Facing>(player).unwrap();
            if let Some(aim_facing) = aim_facing.0 {
                *facing = aim_facing;
            }
            *facing
        };

        let attack_loc: ResourceLocation<AttackResource> = "player/basic_attack".parse().unwrap();

        world.trigger(PlayerAttackEvent {
            entity: player,
            facing,
            attack: attack_loc.clone(),
        });

        match CharacterStateEvent::try_new(
            player,
            &state_capabilities,
            Box::new(Attacking::new(&attack_loc, Duration::from_millis(ATTACK_DURATION))),
            prev_state,
        ) {
            Ok(event) => world.trigger(event),
            Err(_) => {
                error!("Failed to create CharacterStateEvent for Attacking state");
            }
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
    attack: ResourceLocation<AttackResource>,
}

const ATTACK_DURATION: u64 = 350;

fn on_player_attack(
    event: On<PlayerAttackEvent>,
    context: AttackContext,
    mut commands: Commands,
) {
    let Some(attack) = context.attack_registry.get_asset(&event.attack) else {
        error!("Invalid player attack event: attack {} does not exist!", event.attack);
        return;
    };

    let Some(animation) = context.animation_registry.get_resolved_asset(attack.animation()) else {
        warn!("Invalid player attack definition: animation {} does not exist!", attack.animation());
        return;
    };

    let Some(particle_sprite) = context.character_sprite_registry.get_handle(attack.particle_sprite()) else {
        warn!("Invalid player attack definition: particle sprite {} does not exist!", attack.particle_sprite());
        return;
    };

    let particle_atlas = animation.atlas().clone().with_index(event.facing as usize);

    let particle_sprite = Sprite::from_atlas_image(
        particle_sprite,
        particle_atlas,
    );

    let particle_animation = ParticleAnimation::new(
        event.facing as usize * animation.frames(),
        animation.frames(),
        Duration::from_millis(ATTACK_DURATION / animation.frames() as u64),
    );

    commands.trigger(ParticleSpawnEvent::with_parent(
        particle_sprite,
        particle_animation,
        event.entity,
    ));

    commands.trigger(StaminaEvent::new(event.entity, attack.stamina_cost()));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    indicator_ring_layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let indicator_ring_layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 8, 1, None, None);
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let indicator_ring_layout = texture_atlas_layouts.add(indicator_ring_layout);

        Self {
            indicator_ring_layout,
        }
    }
}
