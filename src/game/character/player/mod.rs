//! Player-specific behavior.

use crate::asset_tracking::LoadResource;
use crate::data::ResourceLocation;
use crate::datagen_api::assets::CharacterSpriteResource;
use crate::datagen_api::attack::{AttackContext, AttackResource};
use crate::game::character::assets::CharacterResource;
//use crate::game::object::Shadow;
use crate::game::character::health::Health;
use crate::game::character::stamina::{Stamina, StaminaEvent};
use crate::game::character::{character_bundle, CharacterBuilderContext, Facing};
use crate::game::particle::{ParticleAnimation, ParticleSpawnEvent};
use crate::game::physics::movement::MovementController;
use bevy::prelude::*;
use std::str::FromStr;
use tracing::warn;

mod input;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(input::plugin);

    app.load_resource::<PlayerAssets>();

    app.add_observer(on_aim_facing_changed);
    app.add_observer(on_player_attack);
}

/// The player character.
pub fn player_bundle(
    position: Vec3,
    max_speed: f32,
    player_assets: &PlayerAssets,
    scale: f32,
    context: &CharacterBuilderContext,
) -> impl Bundle {
    let player_data_location =
        ResourceLocation::<CharacterResource>::from_str("player").unwrap();

    let movement_controller = MovementController {
        max_speed,
        ..default()
    };

    let character_data = character_bundle(
        player_data_location,
        position,
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

#[derive(EntityEvent, Debug, Clone, Reflect)]
struct PlayerAttackEvent {
    entity: Entity,
    facing: Facing,
    attack: ResourceLocation<AttackResource>,
}

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
        animation.interval(),
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
