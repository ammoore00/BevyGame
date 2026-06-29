use crate::data::prelude::*;
use crate::game::character::animation::AnimationContext;
use crate::game::character::assets::{CharacterData, CharacterResource};
use crate::game::character::state::states::Idle;
use crate::game::level::grid::coords::WorldPosition;
use crate::game::physics::components::PhysicsData;
use crate::game::physics::movement::{MovementController, DEFAULT_MAX_SPEED};
use crate::screens::Screen;
use crate::{action_state_scene, Scale};
use animation::{AnimationStateMap, CharacterAnimationTracker};
use bevy::ecs::query::{QueryData, QueryItem};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use game_data::prelude::*;
use state::tracking::ActionStateTracker;
use std::any::TypeId;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::{Arc, RwLock};
use game_data::register_prototype_system;
use crate::data::loader::LoaderJobManager;

pub mod animation;
pub mod health;
pub mod player;
pub mod stamina;
pub mod assets;
pub(crate) mod state;
pub mod attack;
pub mod npc;

pub fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<CharacterResource>();

    app.add_plugins((
        animation::plugin,
        assets::plugin,
        attack::plugin,
        health::plugin,
        npc::plugin,
        player::plugin,
        stamina::plugin,
        state::plugin,
    ));

    app.add_systems(Update, initialize_characters.run_if(in_state(Screen::Gameplay)));
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum Facing {
    NorthWest = 0,
    West = 1,
    #[default]
    SouthWest = 2,
    South = 3,
    SouthEast = 4,
    East = 5,
    NorthEast = 6,
    North = 7,
}

impl From<usize> for Facing {
    fn from(index: usize) -> Self {
        match index {
            0 => Self::NorthWest,
            1 => Self::West,
            2 => Self::SouthWest,
            3 => Self::South,
            4 => Self::SouthEast,
            5 => Self::East,
            6 => Self::NorthEast,
            7 => Self::North,
            _ => unreachable!(),
        }
    }
}

impl From<Vec2> for Facing {
    fn from(vec: Vec2) -> Self {
        // Calculate angle in radians (-PI to PI)
        // Note: atan2(z, x) where x is "forward" and z is "right"
        let angle = vec.x.atan2(vec.y);

        // Convert to 0-8 range, where each direction occupies 45 degrees (PI/4 radians)
        // Add PI to shift range from [-PI, PI] to [0, 2*PI]
        // Add PI/8 to center the divisions on the cardinal directions
        // Add 3PI/2 to rotate divisions to align with sprite sheets
        // Divide by PI/4 (45 degrees) to get 0-8 range
        let direction_index = ((angle
            + std::f32::consts::PI
            + std::f32::consts::FRAC_PI_8
            + std::f32::consts::FRAC_PI_2 * 3.0)
            / std::f32::consts::FRAC_PI_4)
            .floor() as i32
            % 8;

        Self::from(direction_index as usize)
    }
}
