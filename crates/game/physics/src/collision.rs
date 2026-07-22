use crate::components::{CollisionContact, PhysicsKind};
use crate::{Collider, PhysicsData};
use bevy::prelude::*;
use common::{AppSystems, GameplaySystems, PausableSystems, TilePosition, WorldPosition};
use std::collections::HashMap;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (check_collisions, validate_colliders)
            .in_set(GameplaySystems)
            .in_set(PausableSystems)
            .in_set(AppSystems::Update),
    );
}

fn validate_colliders(
    query: Query<Entity, (With<PhysicsData>, Without<Collider>)>,
    query2: Query<Entity, (With<Collider>, Without<PhysicsData>)>,
    mut commands: Commands,
) {
    for invalid_entity in query {
        error!(
            "Entity {:?} has PhysicsData but no Collider",
            invalid_entity
        );
        commands.entity(invalid_entity).remove::<PhysicsData>();
    }

    for invalid_entity in query2 {
        error!(
            "Entity {:?} has Collider but no PhysicsData",
            invalid_entity
        );
        commands.entity(invalid_entity).remove::<Collider>();
    }
}

/// Maximum distance for collision checks
/// Entities further than this distance apart will not be checked for collisions
pub const MAX_COLLISION_DISTANCE: f32 = 10.0;

fn check_collisions(
    // Get all entities with colliders, and their physics data for how they should behave
    // WorldPosition is only needed for kinematic entities
    collider_query: Query<(
        Entity,
        &PhysicsData,
        &Collider,
        Option<&WorldPosition>,
        Option<&TilePosition>,
    )>,
    mut commands: Commands,
) {
    // Create collections for holding all detected collisions
    // These will be sent to each entity to process movement for physics collisions,
    // or to process other events for detectors
    //
    // Even if these are empty, they still need to be sent to tell the entity
    // that it can start to process movement
    let mut physics_collisions = HashMap::<Entity, Vec<_>>::new();
    let mut detector_collisions = HashMap::<Entity, Vec<_>>::new();

    // Sort colliders so that the first in the pair is always kinematic,
    // as only kinematic physics objects check for collisions
    let collider_pairs =
        collider_query
            .iter_combinations::<2>()
            .filter_map(|[first, second]| match (first.1, second.1) {
                (PhysicsData::Kinematic { .. }, PhysicsData::Kinematic { .. }) => {
                    Some([first, second])
                }
                (PhysicsData::Kinematic { .. }, _) => Some([first, second]),
                (_, PhysicsData::Kinematic { .. }) => Some([second, first]),
                _ => None,
            });

    for [
        (entity, _, collider, pos, _),
        (other_entity, other_physics_data, other_collider, other_pos, other_tile_pos),
    ] in collider_pairs
    {
        let Some(pos) = pos else {
            error!("Entity with kinematic physics data is missing WorldPosition component");
            continue;
        };

        let other_pos = if let Some(other_pos) = other_pos {
            *other_pos.0
        } else if let Some(other_tile_pos) = other_tile_pos {
            other_tile_pos.0.as_vec3()
        } else {
            error!("Collider entity must have either WorldPosition or TilePosition component");
            continue;
        };

        physics_collisions.entry(entity).or_default();
        detector_collisions.entry(entity).or_default();

        // Filter out entities that are too far apart to improve performance
        // Distance is calculated as Chebyshev, not Euclidean, because it makes
        //  validation easier when creating colliders
        if (*pos.0 - other_pos).abs().max_element() > MAX_COLLISION_DISTANCE {
            continue;
        }

        let Some(contact) = collider.check_collision(other_collider) else {
            continue;
        };

        match *other_physics_data {
            PhysicsData::Detector => {
                let collision = DetectorCollision { _contact: contact };
                detector_collisions
                    .entry(entity)
                    .and_modify(|list| list.push(collision.clone()));
            }
            PhysicsData::Static => {
                let collision = PhysicsCollision {
                    contact,
                    _kind: PhysicsKind::Static,
                };
                physics_collisions
                    .entry(entity)
                    .and_modify(|list| list.push(collision.clone()));
            }
            // If the other is kinematic, it needs to deal with the collision as well,
            // so send it to both objects
            PhysicsData::Kinematic { .. } => {
                let first_collision = PhysicsCollision {
                    contact: contact.clone(),
                    _kind: PhysicsKind::Kinematic,
                };
                physics_collisions
                    .entry(entity)
                    .and_modify(|list| list.push(first_collision.clone()));

                // Collision in the other direction needs an inverted normal vector
                // since this is opposite to the original contact test
                let second_contact = contact.with_inverted_normal();
                let second_collision = PhysicsCollision {
                    contact: second_contact,
                    _kind: PhysicsKind::Kinematic,
                };
                physics_collisions
                    .entry(other_entity)
                    .or_default()
                    .push(second_collision.clone())
            }
        }
    }

    for (colliding_entity, physics_collisions) in physics_collisions {
        commands.trigger(PhysicsCollisionsProcessedEvent {
            entity: colliding_entity,
            physics_collisions,
        })
    }

    for (colliding_entity, detector_collisions) in detector_collisions {
        commands.trigger(DetectorCollisionsProcessedEvent {
            entity: colliding_entity,
            _detector_collisions: detector_collisions,
        })
    }
}

#[derive(EntityEvent, Debug, Clone)]
pub struct PhysicsCollisionsProcessedEvent {
    pub entity: Entity,
    pub physics_collisions: Vec<PhysicsCollision>,
}

#[derive(EntityEvent, Debug, Clone)]
pub struct DetectorCollisionsProcessedEvent {
    pub entity: Entity,
    pub _detector_collisions: Vec<DetectorCollision>,
}

#[derive(Debug, Clone)]
pub struct PhysicsCollision {
    pub contact: CollisionContact,
    pub _kind: PhysicsKind,
}

#[derive(Debug, Clone)]
pub struct DetectorCollision {
    pub _contact: CollisionContact,
}
