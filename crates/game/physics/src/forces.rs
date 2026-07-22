use bevy::prelude::*;
use std::ops::{Add, AddAssign, Sub, SubAssign};
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
    entity: Entity,
    impulse: Impulse,
}

fn on_apply_impulse(impulse: On<ApplyImpulse>, mut query: Query<&mut Velocity>) {
    if let Ok(velocity) = query.get_mut(impulse.entity).map(Mut::into_inner) {
        *velocity += impulse.impulse;
    }
}

/// A one-time impulse to modify an entity's velocity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Impulse(Vec3);
impl From<Vec3> for Impulse {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

/// An entity's current velocity
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Velocity(Vec3);
impl From<Vec3> for Velocity {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

impl Add<Impulse> for Velocity {
    type Output = Self;

    fn add(self, rhs: Impulse) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl AddAssign<Impulse> for Velocity {
    fn add_assign(&mut self, rhs: Impulse) {
        self.0 += rhs.0;
    }
}
impl Sub<Impulse> for Velocity {
    type Output = Self;

    fn sub(self, rhs: Impulse) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl SubAssign<Impulse> for Velocity {
    fn sub_assign(&mut self, rhs: Impulse) {
        self.0 -= rhs.0;
    }
}

impl<T> Add<T> for Velocity
where
    Vec3: Add<T, Output = Vec3>,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl<T> AddAssign<T> for Velocity
where
    Vec3: AddAssign<T>,
{
    fn add_assign(&mut self, rhs: T) {
        self.0 += rhs;
    }
}
impl<T> Sub<T> for Velocity
where
    Vec3: Sub<T, Output = Vec3>,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self(self.0 - rhs)
    }
}
impl<T> SubAssign<T> for Velocity
where
    Vec3: SubAssign<T>,
{
    fn sub_assign(&mut self, rhs: T) {
        self.0 -= rhs;
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

        Self {
            entity,
            is_marker,
        }
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

/// Continuous force applied to an entity.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum Force {
    /// A target velocity to apply to an entity
    Velocity(Vec3),
    /// A continuous acceleration to apply to an entity
    Acceleration {
        /// The magnitude and direction of the acceleration
        value: Vec3,
        /// The minimum speed for the acceleration to slow the entity
        min_speed: f32,
        /// The maximum speed for the acceleration to speed up the entity
        max_speed: f32,
    },
}
