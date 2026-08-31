use assets::resource::characters::{AnimationData, AnimationResource, ResolvedAnimationData};
use bevy::prelude::*;
use common::Facing;
use data::loc::ResourceLocation;
use data::prelude::SystemRegistry;
use std::any::TypeId;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Component, Debug, Clone, Reflect)]
pub struct CharacterAnimationTracker {
    pub default_animation: Handle<AnimationData>,
    pub current_animation: Handle<AnimationData>,
    pub prev_animation: Handle<AnimationData>,

    pub facing: Facing,
    pub timer: Timer,
    pub frame: usize,
}

impl CharacterAnimationTracker {
    pub fn new(default: Handle<AnimationData>, assets: &Assets<AnimationData>) -> Self {
        let frame_data = assets.get(default.id()).unwrap().unwrap().frame_data();
        let interval = frame_data.frame_duration(0).unwrap();

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

        let animation = assets.get(self.current_animation.id()).unwrap();

        self.frame = (self.frame + 1) % animation.unwrap().frame_data().num_frames();
        self.timer.set_duration(
            animation
                .unwrap()
                .frame_data()
                .frame_duration(self.frame)
                .unwrap(),
        );
    }

    fn get_image(&self, assets: &Assets<AnimationData>) -> Handle<Image> {
        assets
            .get(self.current_animation.id())
            .unwrap()
            .unwrap()
            .image()
            .clone()
    }

    fn get_atlas(&self, assets: &Assets<AnimationData>) -> TextureAtlas {
        assets
            .get(self.current_animation.id())
            .unwrap()
            .unwrap()
            .atlas()
            .clone()
    }

    pub(crate) fn get_atlas_index(&self, assets: &Assets<AnimationData>) -> usize {
        self.frame
            + self.facing as usize
                * assets
                    .get(self.current_animation.id())
                    .unwrap()
                    .unwrap()
                    .frame_data()
                    .num_frames()
    }
}

/// Maps characters states to animation data
#[derive(Component, Debug, Clone, Reflect)]
pub struct AnimationStateMap(HashMap<TypeId, Handle<AnimationData>>);
impl AnimationStateMap {
    pub fn new(
        map: &HashMap<TypeId, ResourceLocation<AnimationResource>>,
        context: &SystemRegistry<AnimationResource>,
    ) -> Self {
        let map = map
            .iter()
            .map(|(type_id, location)| (*type_id, context.get_handle(location).unwrap().clone()))
            .collect();
        AnimationStateMap(map)
    }

    pub fn get(&self) -> &HashMap<TypeId, Handle<AnimationData>> {
        &self.0
    }
}

/*
impl AnimationStateMap {
    pub fn _from_resource_location_map(level: &HashMap<TypeId, ResourceLocation<AnimationResource>>, registry: &ResolvedResourceRegistry<AnimationResource>) -> Self {
        let resolved_map = level.iter()
            .level(|(type_id, location)| {
                (*type_id, registry.get(location).unwrap().clone())
            })
            .collect();
        AnimationStateMap(resolved_map)
    }
}
 */
