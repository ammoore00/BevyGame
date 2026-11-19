use bevy::prelude::*;

mod ai;
mod attack;
mod character;
mod animation;

pub use {
    ai::{
        AiBehaviorParams, AiHearingParams, AiParams, AiRegistry, AiResource, AiSenseParams,
        AiSightParams,
    },
    animation::{CharacterAnimationSet, CharacterAnimationList},
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
        attack::plugin,
        character::plugin,
    ));
}
