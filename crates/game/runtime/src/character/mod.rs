use crate::action_state_scene;
use animation::{AnimationStateMap, CharacterAnimationTracker};
use assets::action_states::Idle;
use assets::resource::character::{AnimationContext, CharacterData, CharacterResource};
use bevy::ecs::query::{QueryData, QueryItem};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::{Facing, GameplaySystems, Scale, WorldPosition};
use data::prelude::*;
use data::register_prototype_system;
use physics::{MovementController, PhysicsData, DEFAULT_MAX_SPEED};
use state::ActionStateTracker;
use std::any::TypeId;
use std::fmt::Debug;

pub mod animation;
pub mod health;
pub mod player;
pub mod stamina;
pub mod state;
pub mod npc;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        health::plugin,
        npc::plugin,
        player::plugin,
        stamina::plugin,
        state::plugin,
    ));

    app.add_systems(Update, initialize_characters.in_set(GameplaySystems));
}

register_prototype_system!(initialize_characters, CharacterBuilder);

/// Marker for a fully initialized character. Presence of this Component
/// guarantees inclusion of all other necessary Components
///
/// # Panics
/// This should never be constructed directly and will panic at runtime if done so
///
/// Use the SceneComponent `CharacterPrototype` instead for constructing characters
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Character(()); // Private unit field to prevent improper construction
impl PrototypeFinalizedMarker for Character {
    fn new(_: PrototypeMarkerToken) -> Self { Self(()) }
}

/// Used to temporarily store the location from which to load the character's data
/// when the entity is constructed from its template
///
/// This component should be removed once the character's data has been loaded
///
/// # Errors
/// This component must have a Default implementation to be loaded through BSN,
/// but any attempt to use the default value will result in an error when the
/// character data is loaded
///
/// This will not panic, but it will result in an error, and the character will not be spawned
#[derive(Component, Clone)]
pub struct CharacterDataLocation(ResourceLocation<CharacterResource>);
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

/// SceneComponent used to construct a character
///
/// See `CharacterProps` for parameters
#[derive(SceneComponent, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[scene(CharacterProps)]
pub struct CharacterPrototype;
impl CharacterPrototype {
    /// Initializes character components which do not need top be loaded from data
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
            PhysicsData::kinematic(Vec3::ZERO)
            Facing
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
        mut commands: Commands
    ) -> Result<(), BevyError> {
        let data = context.get_character_data(&data_loc.0).ok_or(BevyError::error("Failed to find character data"))?;

        let animation_context = context.animation_context();
        let animation_map = AnimationStateMap(data.resolve_animation_handles(animation_context));

        let state_capabilities = data.state_capabilities().clone();

        let animations =
            data.resolve_animation_handles(context.animation_context());
        let idle_animation =
            animations.get(&TypeId::of::<Idle>()).cloned()
                .expect("Failed to find idle animation for player character");

        let animation_assets = context.animation_context().resolved_assets();
        let animation_tracker =
            CharacterAnimationTracker::new(idle_animation, animation_assets);
        let sprite = animation_tracker.default_sprite(animation_assets);

        let collider = data.collider().make_collider(position.as_vec3());

        commands.entity(entity).insert((
            animation_tracker,
            animation_map,
            state_capabilities,
            sprite,
            collider,
            Transform::from_scale(Vec3::splat(context.scale.0)),
        ));

        Ok(())
    }
}

#[derive(SystemParam, getset::Getters)]
pub struct CharacterBuilderContext<'w> {
    #[getset(get = "pub")]
    character_registry: SystemRegistry<'w, CharacterResource>,
    #[getset(get = "pub")]
    animation_context: AnimationContext<'w>,
    #[getset(get = "pub")]
    scale: Res<'w, Scale>
}
impl CharacterBuilderContext<'_> {
    pub fn get_character_data(&self, loc: &ResourceLocation<CharacterResource>) -> Option<&CharacterData> {
        self.character_registry.get_asset(loc)
    }
}