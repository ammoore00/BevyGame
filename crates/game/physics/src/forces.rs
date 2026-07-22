use crate::PhysicsData;
use bevy::prelude::*;
use std::slice;
use std::sync::Arc;

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_apply_impulse);
    app.add_observer(on_apply_force);
    app.add_observer(on_remove_force);
}

/// Apply a one-time impulse to an entity.
/// This will be added to the entity's velocity, then cleared.
#[derive(EntityEvent, Debug, Clone, Copy, PartialEq, derive_new::new)]
pub struct ApplyImpulse {
    pub entity: Entity,
    pub impulse: Impulse,
}

fn on_apply_impulse(impulse_event: On<ApplyImpulse>, mut query: Query<&mut PhysicsData>) {
    if let Ok(PhysicsData::Kinematic(kinematic_data)) =
        query.get_mut(impulse_event.entity).map(Mut::into_inner)
    {
        kinematic_data.impulses.push_back(impulse_event.impulse);
    } else {
        error!("Attempted to apply impulse to non-kinematic entity");
    }
}

/// A one-time impulse to modify an entity's velocity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Impulse(pub Vec3);
impl From<Vec3> for Impulse {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

type MarkerApplier = Arc<dyn Fn(EntityCommands) + Send + Sync + 'static>;
type MarkerPredicate = Arc<dyn Fn(&EntityRef) -> bool + Send + Sync + 'static>;

/// Add or modify an existing force on an entity
#[derive(EntityEvent)]
pub struct ApplyForce {
    entity: Entity,
    force: Force,
    apply_marker: MarkerApplier,
    is_marker: MarkerPredicate,
}
impl ApplyForce {
    pub fn new<T>(entity: Entity, force: Force) -> Self
    where
        T: Component + Default,
    {
        let apply_marker = Arc::new(|mut entity: EntityCommands| {
            entity.insert(T::default());
        });

        let is_marker = Arc::new(|entity_ref: &EntityRef| entity_ref.contains::<T>());

        Self {
            entity,
            force,
            apply_marker,
            is_marker,
        }
    }
}

/// Remove a force from an entity
#[derive(EntityEvent)]
pub struct RemoveForce {
    entity: Entity,
    is_marker: MarkerPredicate,
}
impl RemoveForce {
    pub fn new<T>(entity: Entity) -> Self
    where
        T: Component + Default,
    {
        let is_marker = Arc::new(|entity_ref: &EntityRef| entity_ref.contains::<T>());

        Self { entity, is_marker }
    }
}

fn on_apply_force(
    force_event: On<ApplyForce>,
    existing_forces_query: Query<&AppliedForces>,
    mut commands: Commands,
) {
    // Remove any existing forces that match the marker
    remove_force_impl(
        force_event.entity,
        force_event.is_marker.clone(),
        existing_forces_query,
        commands.reborrow(),
    );

    // Add the new force
    let force = commands.spawn(force_event.force).id();
    (force_event.apply_marker)(commands.entity(force));
    commands
        .entity(force_event.entity)
        .add_one_related::<ForcedEntity>(force);
}

fn on_remove_force(
    force_event: On<RemoveForce>,
    existing_forces_query: Query<&AppliedForces>,
    mut commands: Commands,
) {
    remove_force_impl(
        force_event.entity,
        force_event.is_marker.clone(),
        existing_forces_query,
        commands.reborrow(),
    );
}

fn remove_force_impl(
    target: Entity,
    is_marker: MarkerPredicate,
    existing_forces_query: Query<&AppliedForces>,
    mut commands: Commands,
) {
    // Get all existing forces applied to the target entity
    let existing_forces = existing_forces_query
        .get(target)
        .map(|forces| forces.iter().collect::<Vec<Entity>>())
        .unwrap_or_default();

    // Find any forces that already match the marker and remove them
    commands.queue(move |world: &mut World| {
        for force in existing_forces {
            if let Ok(entity_ref) = world.get_entity(force) {
                // Evaluates T::Component presence dynamically without compile-time generics!
                if is_marker(&entity_ref) {
                    world.despawn(force);
                }
            }
        }
    });
}

/// Tracks which entity a force is applied to.
#[derive(Component, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[relationship(relationship_target = AppliedForces)]
pub struct ForcedEntity(pub Entity);

/// Tracks which forces are applied to an entity.
#[derive(Component, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[relationship_target(relationship = ForcedEntity, linked_spawn)]
pub struct AppliedForces(Vec<Entity>);
impl<'a> IntoIterator for &'a AppliedForces {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = slice::Iter<'a, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Continuous force applied to an entity.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum Force {
    /// A target velocity to apply to an entity
    TargetVelocity(TargetVelocity),
    /// A continuous acceleration to apply to an entity
    Acceleration(Vec3),
}
impl Force {
    pub fn target_velocity(target: Vec3) -> Self {
        Self::TargetVelocity(TargetVelocity {
            target,
            ..default()
        })
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct TargetVelocity {
    /// The target velocity
    pub target: Vec3,
    /// Whether to slow down the object if it is moving faster than the target velocity
    /// when the current and target velocity components have the same direction.
    pub can_slow: bool,
    /// If the velocity has components opposite to the object's current velocity,
    /// whether it can cause the object's velocity to cross zero.
    pub zero_crossing: bool,
    /// The acceleration to apply while reaching the target velocity
    /// If None, the velocity will be applied instantaneously.
    pub acceleration: Option<f32>,
    /// Whether to use vector steering or to apply the velocity directly to components
    pub should_steer: bool,
    /// Axes to apply the target velocity on. Defaults to X and Z.
    pub axes: TargetAxes,
}
impl Default for TargetVelocity {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            can_slow: false,
            zero_crossing: true,
            acceleration: None,
            should_steer: true,
            axes: TargetAxes::default(),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetAxes {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}
impl TargetAxes {
    pub const XZ: TargetAxes = Self { x: true, y: false, z: true };
    pub const Y: TargetAxes = Self { x: false, y: true, z: false };
    pub const ALL: TargetAxes = Self { x: true, y: true, z: true };
}
impl Default for TargetAxes {
    fn default() -> Self {
        Self::XZ
    }
}