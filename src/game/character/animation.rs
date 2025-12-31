use crate::game::character::{CharacterStateTracker, Facing};
use crate::screens::Screen;
use crate::{AppSystems, PausableSystems};
use bevy::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<CharacterAnimationData>();

    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (update_animation_state, update_animation_atlas)
                .chain()
                .in_set(AppSystems::Respond),
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

fn update_animation_timer(
    time: Res<Time>,
    assets: Res<Assets<CharacterAnimationData>>,
    mut query: Query<&mut CharacterAnimationTracker>,
) {
    for mut animation in &mut query {
        animation.update_timer(time.delta(), &assets);
    }
}

fn update_animation_state(
    assets: Res<Assets<CharacterAnimationData>>,
    mut query: Query<(
        &CharacterStateTracker,
        &Facing,
        &AnimationStateMap,
        &mut CharacterAnimationTracker,
    )>,
) {
    for (state, facing, map, mut animation) in &mut query {
        let state_id = state.type_id;

        animation.facing = *facing;

        // If the state changed, reset the animation timer/frame based on new map data
        if Some(animation.current.clone()) != map.0.get(&state_id).cloned()
            && let Some(data) = map.0.get(&state_id).cloned()
        {
            let data = assets.get(data.id()).unwrap();

            animation.timer = Timer::new(data.interval, TimerMode::Repeating);
            animation.frame = 0;
            animation.current = map.0.get(&state_id).cloned().unwrap();
        }
    }
}

fn update_animation_atlas(
    assets: Res<Assets<CharacterAnimationData>>,
    mut query: Query<(
        &CharacterStateTracker,
        &CharacterAnimationTracker,
        &AnimationStateMap,
        &mut Sprite,
    )>,
) {
    for (state, animation, map, mut sprite) in &mut query {
        let Some(data) = map.0.get(&state.type_id).cloned() else {
            continue;
        };
        let data = assets.get(data.id()).unwrap();

        sprite.image = data.image.clone();

        let mut atlas = data.atlas.clone();
        // Calculate index: (Direction Row * Frames per row) + Current Frame
        atlas.index = animation.get_atlas_index(&assets);
        sprite.texture_atlas = Some(atlas);
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct CharacterAnimationTracker {
    default: Handle<CharacterAnimationData>,
    current: Handle<CharacterAnimationData>,

    facing: Facing,
    timer: Timer,
    frame: usize,
}

impl CharacterAnimationTracker {
    pub fn new(
        default: Handle<CharacterAnimationData>,
        assets: &Assets<CharacterAnimationData>,
    ) -> Self {
        let interval = assets.get(default.id()).unwrap().interval;

        Self {
            current: default.clone(),
            default,

            facing: Facing::default(),
            timer: Timer::new(interval, TimerMode::Repeating),
            frame: 0,
        }
    }

    pub fn default_sprite(&self, assets: &Assets<CharacterAnimationData>) -> Sprite {
        Sprite::from_atlas_image(
            self.get_image(assets).clone(),
            self.get_atlas(assets).clone(),
        )
    }

    fn update_timer(&mut self, delta: Duration, assets: &Assets<CharacterAnimationData>) {
        self.timer.tick(delta);

        if !self.timer.is_finished() {
            return;
        }

        self.frame = (self.frame + 1) % assets.get(self.current.id()).unwrap().frames;
    }

    fn get_image(&self, assets: &Assets<CharacterAnimationData>) -> Handle<Image> {
        assets.get(self.current.id()).unwrap().image.clone()
    }

    fn get_atlas(&self, assets: &Assets<CharacterAnimationData>) -> TextureAtlas {
        assets.get(self.current.id()).unwrap().atlas.clone()
    }

    fn get_atlas_index(&self, assets: &Assets<CharacterAnimationData>) -> usize {
        self.frame + self.facing as usize * assets.get(self.current.id()).unwrap().frames
    }
}

/// Stores the sprite and frame data for an animation
#[derive(Asset, Debug, Clone, PartialEq, Reflect)]
pub struct CharacterAnimationData {
    pub image: Handle<Image>,
    pub atlas: TextureAtlas,
    pub frames: usize,
    pub interval: Duration,
}

/// Maps character states to animation data
#[derive(Component, Debug, Clone, Reflect)]
pub struct AnimationStateMap(pub HashMap<TypeId, Handle<CharacterAnimationData>>);
