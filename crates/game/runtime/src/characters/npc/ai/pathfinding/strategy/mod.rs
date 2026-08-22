use crate::characters::npc::ai::pathfinding::PathfinderSystems;
use crate::characters::npc::ai::pathfinding::pathfinder::CancelPathing;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use std::any::TypeId;
use std::marker::PhantomData;

pub mod follow;
pub mod wander;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((follow::plugin, wander::plugin));
}

trait PathfindStrategyRegistry {
    fn register_pathfind_strategy<T: PathfindStrategy + Component>(&mut self);
}
impl PathfindStrategyRegistry for App {
    fn register_pathfind_strategy<T: PathfindStrategy + Component>(&mut self) {
        self.add_observer(on_pathfind_strategy_added::<T>);
        self.add_observer(on_pathfind_strategy_removed::<T>);

        self.add_message::<RemoveOtherStrategies<T>>();

        self.add_systems(
            Update,
            process_strategy_messages::<T>.in_set(PathfinderSystems::Update),
        );
    }
}

#[reflect_trait]
pub trait PathfindStrategy: Send + Sync + 'static {}

fn on_pathfind_strategy_added<T: PathfindStrategy + Component>(
    event: On<Add, T>,
    mut message_writer: MessageWriter<RemoveOtherStrategies<T>>,
    mut commands: Commands,
) {
    // Conversion from event to messages used here to allow for exclusive world access,
    // which is necessary for reflection but isn't allowed in the observer system
    message_writer.write(RemoveOtherStrategies::new(event.entity));
    // Store the type of the last inserted strategy so that we don't remove more strategies than we mean to
    commands
        .entity(event.entity)
        .insert(LastStrategyInserted(TypeId::of::<T>()));
}

fn on_pathfind_strategy_removed<T: PathfindStrategy + Component>(
    event: On<Remove, T>,
    mut commands: Commands,
) {
    commands.entity(event.entity).trigger(CancelPathing);
}

#[derive(Message, Debug, PartialEq, Eq, Hash, derive_new::new)]
struct RemoveOtherStrategies<T: PathfindStrategy + Component> {
    entity: Entity,
    _phantom_data: PhantomData<T>,
}
// Manual implementations so that T doesn't need to be Clone or Copy
impl<T: PathfindStrategy + Component> Clone for RemoveOtherStrategies<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PathfindStrategy + Component> Copy for RemoveOtherStrategies<T> {}

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct LastStrategyInserted(TypeId);

fn process_strategy_messages<T: PathfindStrategy + Component>(
    world: &mut World,
    message_reader: &mut SystemState<MessageReader<RemoveOtherStrategies<T>>>,
) {
    let Ok(mut message_reader) = message_reader.get_mut(world) else {
        return;
    };

    let messages = message_reader.read().copied().collect::<Vec<_>>();
    for message in messages {
        if let Ok(last) = world
            .query::<&LastStrategyInserted>()
            .get(world, message.entity)
            && last.0 == TypeId::of::<T>()
        {
            remove_existing_strategies::<T>(message.entity, world);
        } else {
            error!("Failed to get last inserted strategy component!");
        }
    }
}

/// Use reflection to find any existing `PathfindStrategy` components on the entity that do not match the type T and remove them
fn remove_existing_strategies<T: PathfindStrategy + Component>(entity: Entity, world: &mut World) {
    let new_component_id = world.register_component::<T>();

    // Scoped so that immutable reference to type_registry is dropped
    // before we need a mutable reference at the end to remove the components
    let to_remove = {
        let type_registry = world.resource::<AppTypeRegistry>().read();
        let entity_ref = world.entity(entity);

        let mut to_remove = Vec::new();

        // Iterate over components present on the entity archetype
        for component_id in entity_ref.archetype().components() {
            let component_id = *component_id;

            if component_id == new_component_id {
                continue;
            }

            // Get the component and type registration info
            if let Some(info) = world.components().get_info(component_id)
                && let Some(type_id) = info.type_id()
                && let Some(registration) = type_registry.get(type_id)
                // Check if the component type implements ReflectPathfindStrategy
                && registration.data::<ReflectPathfindStrategy>().is_some()
            {
                to_remove.push(component_id);
            }
        }

        to_remove
    };

    for comp_id in to_remove {
        world.entity_mut(entity).remove_by_id(comp_id);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Component, Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
    #[reflect(PathfindStrategy)]
    pub struct TestStrategy;
    impl PathfindStrategy for TestStrategy {}

    #[derive(Component, Default, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
    #[reflect(PathfindStrategy)]
    pub struct TestStrategyTwo;
    impl PathfindStrategy for TestStrategyTwo {}

    fn app() -> App {
        let mut app = App::new();

        app.register_pathfind_strategy::<TestStrategy>();
        app.register_pathfind_strategy::<TestStrategyTwo>();

        app
    }

    #[test]
    fn test_add_strategies() {
        // GIVEN
        // A pathfinder without a current active strategy
        let mut app = app();
        let entity = app.world_mut().spawn_empty().id();

        // WHEN
        // We add a new one
        app.world_mut().entity_mut(entity).insert(TestStrategy);

        // And then run systems
        app.update();

        // THEN
        // The new one should be added
        assert!(app.world().entity(entity).get::<TestStrategy>().is_some());
        // And the other strategy should not be added
        assert!(
            app.world()
                .entity(entity)
                .get::<TestStrategyTwo>()
                .is_none()
        );
    }

    #[test]
    fn test_remove_existing_strategies() {
        // GIVEN
        // A pathfinder with a current active strategy
        let mut app = app();
        let entity = app.world_mut().spawn(TestStrategy).id();
        app.update();

        // WHEN
        // We add a different one
        app.world_mut().entity_mut(entity).insert(TestStrategyTwo);

        // And then run systems
        app.update();

        // THEN
        // The new one should be added
        assert!(
            app.world()
                .entity(entity)
                .get::<TestStrategyTwo>()
                .is_some()
        );
        // And the old one should be removed
        assert!(app.world().entity(entity).get::<TestStrategy>().is_none());
    }

    #[test]
    fn test_multiple_strategies_single_frame() {
        // GIVEN
        // A pathfinder
        let mut app = app();
        let entity = app.world_mut().spawn_empty().id();

        // WHEN
        // We try to add multiple strategies within the same frame
        app.world_mut().entity_mut(entity).insert(TestStrategy);
        app.world_mut().entity_mut(entity).insert(TestStrategyTwo);

        // And then run systems
        app.update();

        // THEN
        // Only the most recent one should be added
        assert!(
            app.world()
                .entity(entity)
                .get::<TestStrategyTwo>()
                .is_some()
        );
        assert!(app.world().entity(entity).get::<TestStrategy>().is_none());
    }
}
