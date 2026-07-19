use crate::character::stamina::StaminaEvent;
use crate::particle::{ParticleAnimation, ParticleSpawnEvent};
use assets::resource::characters::{AttackContext, AttackProgress, AttackResource, KeyFrame};
use bevy::prelude::*;
use common::{AppSystems, Facing, GameplaySystems, PausableSystems, WorldCoords, WorldPosition, marker, offset_position_to_facing};
use data::loc::ResourceLocation;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_attack);

    app.add_systems(
        Update,
        update_attack_key_frames
            .in_set(GameplaySystems)
            .in_set(PausableSystems)
            .in_set(AppSystems::Update),
    );
}

#[derive(EntityEvent, Debug, Clone, Reflect, derive_new::new)]
pub struct AttackEvent {
    entity: Entity,
    facing: Facing,
    attack: ResourceLocation<AttackResource>,
}

fn on_attack(event: On<AttackEvent>, context: AttackContext, mut commands: Commands) {
    let Some(attack) = context.attack_registry.get_asset(&event.attack) else {
        return error!(
            "Invalid attack event: attack {} does not exist!",
            event.attack
        );
    };

    let Ok(animation) = context.animation_context.get_data(attack.animation()) else {
        return error!(
            "Invalid attack definition: animation {} does not exist!",
            attack.animation()
        );
    };

    let Some(particle_sprite) = context
        .character_sprite_registry
        .get_handle(attack.particle_sprite())
    else {
        return error!(
            "Invalid attack definition: particle sprite {} does not exist!",
            attack.particle_sprite()
        );
    };

    let particle_atlas = animation.atlas().clone().with_index(event.facing as usize);
    let particle_sprite = Sprite::from_atlas_image(particle_sprite, particle_atlas);
    let particle_animation = ParticleAnimation::new(
        event.facing as usize * animation.frame_data().num_frames(),
        animation.frame_data().clone(),
    );

    commands
        .entity(event.entity)
        .insert(CurrentAttack::new(event.attack.clone()));

    commands.trigger(ParticleSpawnEvent::with_parent(
        particle_sprite,
        particle_animation,
        event.entity,
    ));

    commands.trigger(StaminaEvent::new(event.entity, attack.stamina_cost()));
}

#[derive(Component, Debug)]
struct CurrentAttack {
    loc: ResourceLocation<AttackResource>,
    progress: AttackProgress,
}
impl CurrentAttack {
    fn new(definition: ResourceLocation<AttackResource>) -> Self {
        Self {
            loc: definition,
            progress: AttackProgress::new(0.0),
        }
    }
}

fn update_attack_key_frames(
    attacking_query: Query<(Entity, &mut CurrentAttack, &WorldPosition, &Facing, &Children)>,
    non_attacking_query: Query<&Children, Without<CurrentAttack>>,
    existing_hitbox_query: Query<Entity, With<AttackHitbox>>,
    attack_context: AttackContext,
    time: Res<Time>,
    mut commands: Commands,
) {
    // TODO: Make this less janky
    for children in non_attacking_query.iter() {
        let existing_hitboxes = existing_hitbox_query.iter_many(children);
        for hitbox_entity in existing_hitboxes {
            commands.entity(hitbox_entity).despawn();
        }
    }

    for (entity, mut attack, pos, facing, children) in attacking_query {
        let Some(definition) = attack_context.attack_registry.get_asset(&attack.loc) else {
            commands.entity(entity).remove::<CurrentAttack>();
            error!("Invalid attack definition: {} does not exist!", attack.loc);
            continue;
        };

        let existing_hitboxes = existing_hitbox_query.iter_many(children);
        for hitbox_entity in existing_hitboxes {
            commands.entity(hitbox_entity).despawn();
        }

        let increment = definition.get_progress_increment(time.delta());

        let progress = (*increment + *attack.progress).min(1.0);
        attack.progress = AttackProgress::new(progress);

        let hitboxes = definition.key_frames().get_active_frames(attack.progress);

        for key_frame in hitboxes {
            let hitbox_entity = commands
                .spawn(attack_hitbox(key_frame, pos.0, *facing, attack.progress))
                .id();
            commands.entity(entity).add_child(hitbox_entity);
        }

        if progress == 1.0 {
            commands.entity(entity).remove::<CurrentAttack>();
        }
    }
}

marker!(pub AttackHitbox);

// TODO: Improve this to not respawn hitboxes every frame
fn attack_hitbox(
    key_frame: &KeyFrame,
    pos: WorldCoords,
    facing: Facing,
    attack_progress: AttackProgress,
) -> impl Bundle {
    // TODO: Change this to handle the error more gracefully
    let frame_progress = key_frame
        .current_progress(attack_progress)
        .expect("Attack progress outside of frame window");
    let hitbox_data = key_frame.get_current_interpolated_hitbox(frame_progress);

    let pos = offset_position_to_facing(pos, *hitbox_data.offset(), facing);
    let collider_codec = hitbox_data.collider();

    let collider = collider_codec.make_collider(pos.0);

    (AttackHitbox, collider, WorldPosition(pos))
}
