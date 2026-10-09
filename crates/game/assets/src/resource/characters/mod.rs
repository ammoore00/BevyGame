use bevy::prelude::*;

mod ai;
mod animation;
mod attack;
mod character;

pub use {
    ai::{
        AiBehaviorParams, AiHearingParams, AiParams, AiRegistry, AiResource, AiSenseParams,
        AiSightParams,
    },
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
    app.add_plugins((
        ai::plugin,
        animation::plugin,
        attack::plugin,
        character::plugin,
    ));
}
