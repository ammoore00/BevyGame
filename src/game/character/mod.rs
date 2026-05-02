use crate::game::character::animation::{AnimationStateMap, CharacterAnimationTracker};
use crate::game::level::grid::coords::WorldPosition;
use crate::game::physics::components::{Collider, PhysicsData};
use bevy::prelude::*;
use std::any::TypeId;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::{Arc, RwLock};
use bevy::ecs::system::SystemParam;
use state::action_states;
use crate::data::loader::LoaderJobManager;
use crate::data::registry::{ResolvedSystemRegistry, SystemRegistry};
use crate::data::ResourceLocation;
use crate::datagen_api::animation::AnimationResource;
use crate::datagen_api::assets::CharacterSpriteResource;
use crate::game::character::assets::{CharacterData, CharacterResource};

pub mod animation;
pub mod health;
pub mod player;
pub mod stamina;
pub mod assets;
mod state;
pub mod attack;

pub fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<CharacterResource>();

    app.add_plugins((
        animation::plugin,
        assets::plugin,
        attack::plugin,
        health::plugin,
        player::plugin,
        stamina::plugin,
        state::plugin,
    ));
}

pub fn character(
    data_loc: ResourceLocation<CharacterResource>,
    position: Vec3,
    sprite: Sprite,
    animation_tracker: CharacterAnimationTracker,
    collider: Collider,
    scale: f32,
    context: &CharacterBuilderContext,
) -> impl Bundle {
    // TODO: Proper error handling
    let data = context.get_character_data(&data_loc)
        .unwrap_or_else(|| panic!("Failed to find character data for {}", data_loc));

    let animation_registry = context.animation_registry().resolved_registry();
    let animation_map = AnimationStateMap(data.resolve_animation_handles(animation_registry));

    let state_capabilities = data.state_capabilities().clone();
    
    (
        Character,
        state::action_state(action_states::Idle),
        state_capabilities,
        Facing::default(),
        // Physics
        WorldPosition(position.into()),
        PhysicsData::kinematic(Vec3::ZERO),
        collider,
        // Rendering
        Transform::from_scale(Vec3::splat(scale)),
        sprite,
        animation_tracker,
        animation_map,
    )
}

#[derive(SystemParam, getset::Getters)]
pub struct CharacterBuilderContext<'w> {
    #[getset(get = "pub")]
    character_registry: SystemRegistry<'w, CharacterResource>,
    #[getset(get = "pub")]
    animation_registry: ResolvedSystemRegistry<'w, AnimationResource>,
    #[getset(get = "pub")]
    sprite_registry: SystemRegistry<'w, CharacterSpriteResource>,
}
impl CharacterBuilderContext<'_> {
    pub fn get_character_data(&self, loc: &ResourceLocation<CharacterResource>) -> Option<&CharacterData> {
        self.character_registry.get_asset(loc)
    }
}

#[derive(Component, Asset, Clone, Copy, Reflect)]
pub struct Character;

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
