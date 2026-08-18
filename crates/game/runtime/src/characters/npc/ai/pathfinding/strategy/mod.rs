use crate::characters::npc::ai::pathfinding::PathfinderSystems;
use crate::characters::npc::ai::pathfinding::pathfinder::CancelPathing;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use std::marker::PhantomData;

pub mod follow;
pub mod wander;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((follow::plugin, wander::plugin));
}

trait PathfindStrategyRegistry {
    fn register_pathfind_strategy<T: PathfindStrategy + Component + Copy>(&mut self);
}
impl PathfindStrategyRegistry for App {
    fn register_pathfind_strategy<T: PathfindStrategy + Component + Copy>(&mut self) {
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
) {
    message_writer.write(RemoveOtherStrategies::new(event.entity));
}

fn on_pathfind_strategy_removed<T: PathfindStrategy + Component>(
    event: On<Remove, T>,
    mut commands: Commands,
) {
    commands.entity(event.entity).trigger(CancelPathing);
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq, Hash, derive_new::new)]
struct RemoveOtherStrategies<T: PathfindStrategy + Component> {
    entity: Entity,
    _phantom_data: PhantomData<T>,
}

fn process_strategy_messages<T: PathfindStrategy + Component + Copy>(
    world: &mut World,
    message_reader: &mut SystemState<MessageReader<RemoveOtherStrategies<T>>>,
) {
    let Ok(mut message_reader) = message_reader.get_mut(world) else {
        return;
    };

    let messages = message_reader.read().copied().collect::<Vec<_>>();
    for message in messages {
        remove_existing_strategies::<T>(message.entity, world);
    }
}

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
