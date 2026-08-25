use crate::action_state_scene;
use crate::debug::{Health, Player};
use animation::{AnimationStateMap, CharacterAnimationTracker};
use assets::action_states::Idle;
use assets::resource::characters::{AnimationContext, CharacterData, CharacterResource};
use bevy::ecs::query::{QueryData, QueryItem};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::{marker, Facing, GameplaySystems, Scale, WorldPosition};
use data::prelude::*;
use data::register_prototype_system;
use physics::{DEFAULT_MAX_SPEED, HasGravity, MovementController, PhysicsData};
use state::ActionStateTracker;
use std::any::TypeId;
use std::fmt::Debug;

// TODO: Remove these pub(crate) declarations in favor of better exports
pub(crate) mod animation;
pub(crate) mod attack;
pub(crate) mod health;
pub(crate) mod npc;
pub mod player;
pub(crate) mod stamina;
pub(crate) mod state;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        attack::plugin,
        health::plugin,
        npc::plugin,
        player::plugin,
        stamina::plugin,
        state::plugin,
    ));

    app.add_systems(PreUpdate, remove_dead_entities);
    app.add_systems(Update, initialize_characters.in_set(GameplaySystems));

    app.add_observer(on_death);
}

register_prototype_system!(initialize_characters, CharacterBuilder);

/// Marker for a fully initialized characters. Presence of this Component
/// guarantees inclusion of all other necessary Components
///
/// # Panics
/// This should never be constructed directly and will panic at runtime if done so
///
/// Use the SceneComponent `CharacterPrototype` instead for constructing characters
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Character(()); // Private unit field to prevent improper construction
impl PrototypeFinalizedMarker for Character {
    fn new(_: PrototypeMarkerToken) -> Self {
        Self(())
    }
}

/// Used to temporarily store the location from which to load the characters's data
/// when the entity is constructed from its template
///
/// This component should be removed once the characters's data has been loaded
///
/// # Errors
/// This component must have a Default implementation to be loaded through BSN,
/// but any attempt to use the default value will result in an error when the
/// characters data is loaded
///
/// This will not panic, but it will result in an error, and the characters will not be spawned
#[derive(Component, Clone)]
pub(crate) struct CharacterDataLocation(ResourceLocation<CharacterResource>);
impl Default for CharacterDataLocation {
    fn default() -> Self {
        Self(loc::<CharacterResource>("placeholder").unwrap())
    }
}
impl From<CharacterDataLocation> for ResourceLocation<CharacterResource> {
    fn from(loc: CharacterDataLocation) -> Self {
        loc.0
    }
}

pub struct CharacterProps {
    position: Vec3,
    max_speed: f32,
    data_loc: ResourceLocation<CharacterResource>,
}
impl Default for CharacterProps {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            max_speed: DEFAULT_MAX_SPEED,
            data_loc: loc::<CharacterResource>("placeholder").unwrap(),
        }
    }
}

/// SceneComponent used to construct a characters
///
/// See `CharacterProps` for parameters
#[derive(SceneComponent, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[scene(CharacterProps)]
pub(crate) struct CharacterPrototype;
impl CharacterPrototype {
    /// Initializes characters components which do not need top be loaded from data
    /// Remaining components are loaded from data after spawning (see `initialize_characters`)
    fn scene(props: CharacterProps) -> impl Scene {
        let state = action_state_scene!(Idle);
        bsn! [
            CharacterPrototype
            MovementController {
                // TODO: Move this into Character Data
                max_speed: {props.max_speed},
            }
            WorldPosition({props.position})
            CharacterDataLocation({props.data_loc})
            PhysicsData::kinematic()
            HasGravity
            Facing
            Health
            state
        ]
    }
}
impl Prototype for CharacterPrototype {
    type Marker = Character;
    type Resource = CharacterResource;
    type DataLocation = CharacterDataLocation;
}

struct CharacterBuilder;
impl PrototypeBuilder for CharacterBuilder {
    type Proto = CharacterPrototype;
    type Context<'w, 's> = CharacterBuilderContext<'w>;
    type QueryData<'w, 's> = &'s WorldPosition;

    fn build(
        entity: Entity,
        data_loc: &<Self::Proto as Prototype>::DataLocation,
        position: &QueryItem<'_, '_, <Self::QueryData<'_, '_> as QueryData>::ReadOnly>,
        context: &mut Self::Context<'_, '_>,
        mut commands: Commands,
    ) -> Result<(), BevyError> {
        let data = context
            .get_character_data(&data_loc.0)
            .ok_or(BevyError::error("Failed to find characters data"))?;

        let animation_context = context.animation_context();
        let animation_map = AnimationStateMap(data.resolve_animation_handles(animation_context));

        let state_capabilities = data.state_capabilities().clone();

        let animations = data.resolve_animation_handles(context.animation_context());
        let idle_animation = animations
            .get(&TypeId::of::<Idle>())
            .cloned()
            .expect("Failed to find idle animation for player characters");

        let animation_assets = context.animation_context().resolved_assets();
        let animation_tracker = CharacterAnimationTracker::new(idle_animation, animation_assets);
        let sprite = animation_tracker.default_sprite(animation_assets);

        let collider = data.collider().make_collider(position.as_vec3());

        commands.entity(entity).insert((
            animation_tracker,
            animation_map,
            state_capabilities,
            sprite,
            collider,
            //Transform::from_scale(Vec3::splat(context.scale.0)),
            Transform::from_scale(Vec3::splat(context.scale.0 * 2.)), //TODO: Temporary rescale
        ));

        Ok(())
    }
}

#[derive(SystemParam, getset::Getters)]
struct CharacterBuilderContext<'w> {
    #[getset(get = "pub")]
    character_registry: SystemRegistry<'w, CharacterResource>,
    #[getset(get = "pub")]
    animation_context: AnimationContext<'w>,
    #[getset(get = "pub")]
    scale: Res<'w, Scale>,
}
impl CharacterBuilderContext<'_> {
    pub fn get_character_data(
        &self,
        loc: &ResourceLocation<CharacterResource>,
    ) -> Option<&CharacterData> {
        self.character_registry.get_asset(loc)
    }
}

marker!(pub Dead);

#[derive(EntityEvent, Debug, Clone)]
pub struct DeathEvent {
    entity: Entity,
}

fn on_death(
    event: On<DeathEvent>,
    query: Query<Option<&Player>, With<Character>>,
    mut commands: Commands
) {
    let Ok(player) = query.get(event.entity) else {
        error!("Failed to get death event entity");
        return;
    };

    if let Some(_player) = player {
        // TODO: Player death
    } else {
        // TODO: Make this more sophisticated
        commands.entity(event.entity).insert(Dead);
    }
}

fn remove_dead_entities(
    query: Query<Entity, With<Dead>>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}