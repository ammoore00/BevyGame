use bevy::prelude::*;

mod animation;
mod attack;
mod character;

pub use {
    animation::{
        AnimationData, AnimationRegistry, AnimationResource, FrameData, ResolvedAnimationData,
    },
    attack::{
        AttackContext, AttackDefinition, AttackProgress, AttackRegistry, AttackResource, AttackSet,
        AttackSetRegistry, AttackSetResource, ExclusionGroup, KeyFrame,
    },
    character::{
        CharacterData, CharacterRegistry, CharacterResource, CharacterSpriteRegistry,
        CharacterSpriteResource,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((animation::plugin, attack::plugin, character::plugin));
}
