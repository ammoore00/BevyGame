use std::slice;
use crate::characters::health::{AddIFrames, HealthEvent};
use crate::characters::stamina::StaminaEvent;
use crate::particle::{ParticleAnimation, ParticleSpawnEvent};
use assets::resource::characters::{AttackContext, AttackProgress, AttackResource, KeyFrame};
use bevy::prelude::*;
use common::{
    AppSystems, Facing, GameplaySystems, PausableSystems, WorldCoords, WorldPosition,
    offset_position_to_facing,
};
use data::loc::ResourceLocation;
use physics::{Collider, DetectorCollisionResponse, DetectorCollisionsProcessedMessage, PhysicsData};
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_attack_key_frames
                .in_set(AppSystems::TickTimers)
                .in_set(GameplaySystems)
                .in_set(PausableSystems),
            process_attack_hits.in_set(DetectorCollisionResponse),
        ),
    );

    app.add_observer(on_attack);
}

/// The length of time that an attack will prevent the character from taking additional damage.
pub const ON_HIT_IFRAMES: Duration = Duration::from_millis(250);

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
    attacking_query: Query<(
        Entity,
        &mut CurrentAttack,
        &WorldPosition,
        &Facing,
        Option<&ActiveHitboxes>,
    )>,
    non_attacking_query: Query<&ActiveHitboxes, Without<CurrentAttack>>,
    mut existing_hitbox_query: Query<(Entity, &mut AttackHitbox, &mut Collider, &mut WorldPosition), Without<CurrentAttack>>,
    attack_context: AttackContext,
    time: Res<Time>,
    mut commands: Commands,
) {
    // TODO: Make this less janky
    for hitboxes in non_attacking_query.iter() {
        let existing_hitboxes = existing_hitbox_query.iter_many(hitboxes);
        for hitbox_data in existing_hitboxes {
            commands.entity(hitbox_data.0).despawn();
        }
    }

    for (entity, mut attack, pos, facing, hitboxes) in attacking_query {
        let Some(definition) = attack_context.attack_registry.get_asset(&attack.loc) else {
            commands.entity(entity).remove::<CurrentAttack>();
            error!("Invalid attack definition: {} does not exist!", attack.loc);
            continue;
        };

        let increment = definition.get_progress_increment(time.delta());
        let progress = (*increment + *attack.progress).min(1.0);
        attack.progress = AttackProgress::new(progress);

        let active_key_frames = definition.key_frames().get_active_frames(attack.progress);
        let mut next_hitboxes = Vec::new();
        let current_hitbox_entities = hitboxes.map(|h| h.0.clone()).unwrap_or_default();

        for key_frame in active_key_frames {
            let mut found_existing = None;
            for &hitbox_entity in &current_hitbox_entities {
                if let Ok((_, hitbox, _, _)) = existing_hitbox_query.get(hitbox_entity)
                    && hitbox.key_frame.index() == key_frame.index()
                {
                    found_existing = Some(hitbox_entity);
                    break;
                }
            }

            let frame_progress = key_frame
                .current_progress(attack.progress)
                .expect("Attack progress outside of frame window");
            let hitbox_data = key_frame.get_current_interpolated_hitbox(frame_progress);

            let attack_pos = offset_position_to_facing(pos.0, *hitbox_data.offset(), *facing);
            let collider_codec = hitbox_data.collider();
            let collider = collider_codec.make_collider(attack_pos.0);

            if let Some(hitbox_entity) = found_existing {
                // Update in-place
                if let Ok((_, mut hitbox, mut existing_collider, mut existing_pos)) =
                    existing_hitbox_query.get_mut(hitbox_entity)
                {
                    // Refresh the stored keyframe (though the index is the same, duration progress changed)
                    hitbox.key_frame = key_frame.clone();
                    *existing_collider = collider;
                    *existing_pos = WorldPosition(attack_pos);
                }
                next_hitboxes.push(hitbox_entity);
            } else {
                // Spawn a new hitbox
                let new_hitbox_entity = commands
                    .spawn((
                        AttackHitbox {
                            key_frame: key_frame.clone(),
                            interacted_entities: Vec::new(),
                        },
                        collider,
                        WorldPosition(attack_pos),
                        PhysicsData::Detector,
                    ))
                    .id();

                commands
                    .entity(entity)
                    .add_one_related::<HitboxOwner>(new_hitbox_entity);
                next_hitboxes.push(new_hitbox_entity);
            }
        }

        // Despawn any hitboxes that are no longer active
        for &hitbox_entity in &current_hitbox_entities {
            if !next_hitboxes.contains(&hitbox_entity) {
                commands.entity(hitbox_entity).despawn();
            }
        }

        if progress == 1.0 {
            commands.entity(entity).remove::<CurrentAttack>();
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[relationship(relationship_target = ActiveHitboxes)]
struct HitboxOwner(Entity);

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
#[relationship_target(relationship = HitboxOwner, linked_spawn)]
struct ActiveHitboxes(Vec<Entity>);
impl<'a> IntoIterator for &'a ActiveHitboxes {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = slice::Iter<'a, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Component, Debug, Clone)]
pub struct AttackHitbox {
    key_frame: KeyFrame,
    interacted_entities: Vec<Entity>,
}

fn process_attack_hits(
    mut reader: MessageReader<DetectorCollisionsProcessedMessage>,
    mut attack_hitbox_query: Query<(&mut AttackHitbox, &HitboxOwner)>,
    mut commands: Commands,
) {
    for collision_message in reader.read() {
        for collision in &collision_message.detector_collisions {
            // Filter detector collisions to only attack hitboxes
            let Ok((mut attack_hitbox, owner)) =
                attack_hitbox_query.get_mut(collision.detector_entity)
            else {
                continue;
            };

            // Do not collide with self or entities that have already been hit
            if collision_message.colliding_entity == owner.0
                || attack_hitbox
                .interacted_entities
                .contains(&collision_message.colliding_entity)
            {
                continue;
            }

            attack_hitbox
                .interacted_entities
                .push(collision_message.colliding_entity);

            let key_frame = &attack_hitbox.key_frame;

            if !key_frame.disable_on_hit_iframes() {
                commands.trigger(AddIFrames::new(collision_message.colliding_entity, ON_HIT_IFRAMES));
            }

            commands.trigger(HealthEvent::new(
                collision_message.colliding_entity,
                *key_frame.health_event(),
            ));

            info!("Attack hit!");
        }
    }
}
