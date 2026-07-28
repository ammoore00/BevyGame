use crate::characters::health::{AddIFrames, HealthEvent};
use crate::characters::stamina::StaminaEvent;
use crate::particle::{ParticleAnimation, ParticleSpawnEvent};
use assets::resource::characters::{
    AttackContext, AttackDefinition, AttackProgress, AttackResource, ExclusionGroup, KeyFrame,
};
use bevy::prelude::*;
use common::{
    AppSystems, Facing, GameplaySystems, PausableSystems, WorldCoords, WorldPosition,
    offset_position_to_facing,
};
use data::loc::ResourceLocation;
use physics::{
    Collider, DetectorCollisionResponse, DetectorCollisionsProcessedMessage, PhysicsData,
};
use std::collections::HashMap;
use std::slice;
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

/// Initiate an attack from the entity to the provided facing
#[derive(EntityEvent, Debug, Clone, Reflect, derive_new::new)]
pub struct AttackEvent {
    entity: Entity,
    facing: Facing,
    attack: ResourceLocation<AttackResource>,
}

/// Initiate attack in response to an attack event request
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

    commands.entity(event.entity).insert(CurrentAttack::new(
        event.attack.clone(),
        attack.exclusion_groups(),
    ));

    commands.trigger(ParticleSpawnEvent::with_parent(
        particle_sprite,
        particle_animation,
        event.entity,
    ));

    commands.trigger(StaminaEvent::new(event.entity, attack.stamina_cost()));
}

/// Store the current attack definition location, and the current progress value
#[derive(Component, Debug)]
struct CurrentAttack {
    loc: ResourceLocation<AttackResource>,
    progress: AttackProgress,
    interacted_entities: HashMap<ExclusionGroup, Vec<Entity>>,
}
impl CurrentAttack {
    fn new(
        definition: ResourceLocation<AttackResource>,
        exclusion_groups: &[ExclusionGroup],
    ) -> Self {
        Self {
            loc: definition,
            progress: AttackProgress::new(0.0),
            interacted_entities: exclusion_groups
                .iter()
                .map(|exclusion_group| (exclusion_group.clone(), Vec::new()))
                .collect(),
        }
    }
}

/// Update hitboxes for any ongoing attacks
fn update_attack_key_frames(
    attacking_query: Query<(
        Entity,
        &mut CurrentAttack,
        &WorldPosition,
        &Facing,
        Option<&ActiveHitboxes>,
    )>,
    non_attacking_query: Query<&ActiveHitboxes, Without<CurrentAttack>>,
    mut existing_hitbox_query: Query<
        (Entity, &mut AttackHitbox, &mut Collider, &mut WorldPosition),
        Without<CurrentAttack>,
    >,
    attack_context: AttackContext,
    time: Res<Time>,
    mut commands: Commands,
) {
    // Clean up any potential orphaned hitboxes
    for hitboxes in non_attacking_query.iter() {
        let existing_hitboxes = existing_hitbox_query.iter_many(hitboxes);
        for hitbox_data in existing_hitboxes {
            commands.entity(hitbox_data.0).despawn();
        }
    }

    for (entity, mut attack, pos, facing, hitboxes) in attacking_query {
        // Get the definition for the current attack
        let Some(definition) = attack_context.attack_registry.get_asset(&attack.loc) else {
            commands.entity(entity).remove::<CurrentAttack>();
            error!("Invalid attack definition: {} does not exist!", attack.loc);
            continue;
        };

        let increment = definition.get_progress_increment(time.delta());
        let progress = (*increment + *attack.progress).min(1.0);
        attack.progress = AttackProgress::new(progress);

        // Despawn hitboxes when attack is complete
        if progress == 1.0 {
            let current_hitbox_entities = hitboxes.map(|h| h.0.clone()).unwrap_or_default();
            for hitbox_entity in current_hitbox_entities {
                commands.entity(hitbox_entity).despawn();
            }
            commands.entity(entity).remove::<CurrentAttack>();
            continue;
        }

        // Get current active hitboxes as well as keyframes for the current frame
        let active_key_frames = definition.key_frames().get_active_frames(attack.progress);
        let mut next_hitboxes = Vec::new();
        let current_hitbox_entities = hitboxes.map(|h| h.0.clone()).unwrap_or_default();

        // For each active key frame, modify or create hitboxes as needed
        for key_frame in active_key_frames {
            // Look for an existing hitbox for this key frame
            let mut found_existing = None;
            for &hitbox_entity in &current_hitbox_entities {
                if let Ok((_, hitbox, _, _)) = existing_hitbox_query.get(hitbox_entity)
                    && hitbox.key_frame.index() == key_frame.index()
                {
                    found_existing = Some(hitbox_entity);
                    break;
                }
            }

            // Get the hitbox data for the current instant
            let frame_progress = key_frame
                .current_progress(attack.progress)
                .expect("Attack progress outside of frame window");
            let hitbox_data = key_frame.get_current_interpolated_hitbox(frame_progress);

            let attack_pos = offset_position_to_facing(pos.0, *hitbox_data.offset(), *facing);
            let collider_codec = hitbox_data.collider();
            let collider = collider_codec.make_collider(attack_pos.0);

            // Modify or spawn a new hitbox
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
    }
}

/// Store the owner of the current attack hitboxes
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[relationship(relationship_target = ActiveHitboxes)]
struct HitboxOwner(Entity);

/// Store all active attack hitboxes for the entity
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
}

/// Process collision events for attack hitboxes
fn process_attack_hits(
    mut reader: MessageReader<DetectorCollisionsProcessedMessage>,
    attack_hitbox_query: Query<(&AttackHitbox, &HitboxOwner)>,
    mut attack_owner_query: Query<&mut CurrentAttack>,
    mut commands: Commands,
) {
    for collision_message in reader.read() {
        for collision in &collision_message.detector_collisions {
            // Filter detector collisions to only attack hitboxes
            let Ok((attack_hitbox, owner)) = attack_hitbox_query.get(collision.detector_entity)
            else {
                continue;
            };

            let Ok(mut current_attack) = attack_owner_query.get_mut(owner.0) else {
                error!("Failed to get current attack for hitbox owner!");
                continue;
            };

            // Do not collide with self
            if collision_message.colliding_entity == owner.0 {
                continue;
            }

            let key_frame = &attack_hitbox.key_frame;

            let Some(interacted_entities) = current_attack
                .interacted_entities
                .get_mut(key_frame.exclusion_group())
            else {
                error!(
                    "Unregistered exclusion group {:?} for attack {:?}!",
                    key_frame.exclusion_group(),
                    current_attack.loc
                );
                continue;
            };

            // Do not collide with entities that have already been hit
            if interacted_entities.contains(&collision_message.colliding_entity) {
                continue;
            }
            interacted_entities.push(collision_message.colliding_entity);

            if !key_frame.disable_on_hit_iframes() {
                commands.trigger(AddIFrames::new(
                    collision_message.colliding_entity,
                    ON_HIT_IFRAMES,
                ));
            }

            commands.trigger(HealthEvent::new(
                collision_message.colliding_entity,
                *key_frame.health_event(),
            ));

            info!("Attack hit!");
        }
    }
}
