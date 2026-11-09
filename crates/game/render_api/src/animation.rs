use assets::resource::characters::AnimationResource;
use bevy::prelude::*;
use data::loc::ResourceLocation;

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
pub struct PlayAnimation {
    pub entity: Entity,
    pub animation: ResourceLocation<AnimationResource>,
    pub settings: AnimationSettings,
}
impl PlayAnimation {
    /// Plays a gameplay animation which will pause if the game is paused
    pub fn gameplay(entity: Entity, animation: ResourceLocation<AnimationResource>) -> Self {
        Self {
            entity,
            animation,
            settings: AnimationSettings::default_gameplay(),
        }
    }
    
    /// Plays a UI animation which will play even if gameplay is paused
    pub fn ui(entity: Entity, animation: ResourceLocation<AnimationResource>) -> Self {
        Self {
            entity,
            animation,
            settings: AnimationSettings::default_ui(),
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct AnimationSettings {
    /// Whether the animation should repeat
    pub repeat: bool,
    
    /// If the same animation is already playing, should it restart?
    pub restart_if_present: bool,
    
    /// Whether the animation should pause if gameplay is paused
    pub pause_on_game_pause: bool,
}
impl AnimationSettings {
    pub const fn default_gameplay() -> Self {
        Self {
            repeat: true,
            restart_if_present: false,
            pause_on_game_pause: true,
        }
    }
    
    pub const fn default_ui() -> Self {
        Self {
            repeat: false,
            restart_if_present: false,
            pause_on_game_pause: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnimationKind {
    Gameplay,
    UI,
}

#[derive(EntityEvent, Debug, Clone)]
pub struct StopAnimation {
    pub entity: Entity,
    pub filter: Option<ResourceLocation<AnimationResource>>,
}
impl StopAnimation {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            filter: None,
        }
    }
    
    pub fn new_filtered(entity: Entity, filter: ResourceLocation<AnimationResource>) -> Self {
        Self {
            entity,
            filter: Some(filter),
        }
    }
}
