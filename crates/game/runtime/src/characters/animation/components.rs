use assets::codec::CharacterAnimationSetCodec;
use assets::resource::{AnimationData, AnimationResource};
use bevy::prelude::*;
use common::Facing;
use data::prelude::SystemRegistry;
use enum_map::EnumMap;
use std::any::TypeId;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Component, Debug, Clone)]
pub struct CharacterAnimationTracker {
    pub default_animation: CharacterAnimationSet,
    pub current_animation: CharacterAnimationSet,
    pub prev_animation: CharacterAnimationSet,

    pub facing: Facing,
    pub timer: Timer,
    pub frame: usize,
}

impl CharacterAnimationTracker {
    pub fn new(default: CharacterAnimationSet, assets: &Assets<AnimationData>) -> Self {
        let frame_data = assets
            .get(default.get(Facing::default()).id())
            .unwrap()
            .resolved()
            .frame_data();
        let interval = frame_data.get_frame_duration(0).unwrap();

        Self {
            current_animation: default.clone(),
            prev_animation: default.clone(),
            default_animation: default,

            facing: Facing::default(),
            timer: Timer::new(interval, TimerMode::Repeating),
            frame: 0,
        }
    }

    pub fn default_sprite(&self, assets: &Assets<AnimationData>) -> Sprite {
        Sprite::from_atlas_image(
            self.get_image(assets).clone(),
            self.get_atlas(assets).clone(),
        )
    }

    pub(crate) fn update_timer(&mut self, delta: Duration, assets: &Assets<AnimationData>) {
        self.timer.tick(delta);

        if !self.timer.is_finished() {
            return;
        }

        let animation = assets
            .get(self.current_animation.get(self.facing).id())
            .unwrap();

        self.frame = (self.frame + 1) % animation.resolved().frame_data().num_frames();
        self.timer.set_duration(
            animation
                .resolved()
                .frame_data()
                .get_frame_duration(self.frame)
                .unwrap(),
        );
    }

    fn get_image(&self, assets: &Assets<AnimationData>) -> Handle<Image> {
        assets
            .get(self.current_animation.get(self.facing).id())
            .unwrap()
            .resolved()
            .image()
            .clone()
    }

    fn get_atlas(&self, assets: &Assets<AnimationData>) -> TextureAtlas {
        assets
            .get(self.current_animation.get(self.facing).id())
            .unwrap()
            .resolved()
            .atlas()
            .clone()
    }

    pub(crate) fn get_atlas_index(&self, assets: &Assets<AnimationData>) -> usize {
        self.frame
            + self.facing as usize
                * assets
                    .get(self.current_animation.get(self.facing).id())
                    .unwrap()
                    .resolved()
                    .frame_data()
                    .num_frames()
    }
}

/// Maps characters states to animation data
#[derive(Component, Debug, Clone)]
pub struct AnimationStateMap(HashMap<TypeId, CharacterAnimationSet>);
impl AnimationStateMap {
    pub fn new(
        map: &HashMap<TypeId, CharacterAnimationSetCodec>,
        context: &SystemRegistry<AnimationResource>,
    ) -> Self {
        let map = map
            .iter()
            //.map(|(type_id, location)| (*type_id, context.get_handle(location).unwrap().clone()))
            .map(|(type_id, set)| {
                (
                    *type_id,
                    // TODO: Proper error handling, since this is something that could originate from data
                    CharacterAnimationSet::try_from_codec(set, context).unwrap(),
                )
            })
            .collect();
        AnimationStateMap(map)
    }

    pub fn get(&self) -> &HashMap<TypeId, CharacterAnimationSet> {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterAnimationSet(EnumMap<Facing, Handle<AnimationData>>);
impl CharacterAnimationSet {
    fn try_from_codec(
        codec: &CharacterAnimationSetCodec,
        context: &SystemRegistry<AnimationResource>,
    ) -> Result<Self, AnimationSetError> {
        let map = codec
            .0
            .iter()
            .map(|(facing, loc)| {
                let handle = context
                    .get_handle(loc)
                    .ok_or(AnimationSetError::FailedToGetHandle)?
                    .clone();
                
                Ok((facing, handle))
            })
            .collect::<Result<EnumMap<Facing, Handle<AnimationData>>, AnimationSetError>>()?;
        
        Ok(Self(map))
    }

    pub fn get(&self, facing: Facing) -> Handle<AnimationData> {
        self.0[facing].clone()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
enum AnimationSetError {
    #[error("Failed to get handle for animation data")]
    FailedToGetHandle,
}
