use crate::{data, define_sprite_resource};
use crate::game::character::Facing;
use crate::screens::Screen;
use crate::{define_resource, AppSystems, PausableSystems, StartupSystems};
use bevy::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::data::registry::{SystemRegistry, SystemRegistryMut};
use crate::data::{ResourceFileType, ResourceLocation};
use crate::data::loader::{LoaderJobManager, RonAssetLoader};
use crate::data::sprite::TextureAtlasCodec;
use crate::game::character::state::CharacterStateTracker;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<AnimationAsset>();
    app.init_asset::<ResolvedAnimationData>();
    app.init_asset_loader::<RonAssetLoader<AnimationCodec, AnimationAsset>>();
    app.add_registry_with_discovery::<AnimationResource>();
    app.add_registry_with_discovery::<AnimationSpriteResource>();

    app.add_systems(
        Startup,
        resolve_animation_data.in_set(StartupSystems::ResolveAssets)
    );

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
    assets: Res<Assets<ResolvedAnimationData>>,
    mut query: Query<&mut CharacterAnimationTracker>,
) {
    for mut animation in &mut query {
        animation.update_timer(time.delta(), &assets);
    }
}

fn update_animation_state(
    assets: Res<Assets<ResolvedAnimationData>>,
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
    assets: Res<Assets<ResolvedAnimationData>>,
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
    default: Handle<ResolvedAnimationData>,
    current: Handle<ResolvedAnimationData>,

    facing: Facing,
    timer: Timer,
    frame: usize,
}

impl CharacterAnimationTracker {
    pub fn new(
        default: Handle<ResolvedAnimationData>,
        assets: &Assets<ResolvedAnimationData>,
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

    pub fn default_sprite(&self, assets: &Assets<ResolvedAnimationData>) -> Sprite {
        Sprite::from_atlas_image(
            self.get_image(assets).clone(),
            self.get_atlas(assets).clone(),
        )
    }

    fn update_timer(&mut self, delta: Duration, assets: &Assets<ResolvedAnimationData>) {
        self.timer.tick(delta);

        if !self.timer.is_finished() {
            return;
        }

        self.frame = (self.frame + 1) % assets.get(self.current.id()).unwrap().frames;
    }

    fn get_image(&self, assets: &Assets<ResolvedAnimationData>) -> Handle<Image> {
        assets.get(self.current.id()).unwrap().image.clone()
    }

    fn get_atlas(&self, assets: &Assets<ResolvedAnimationData>) -> TextureAtlas {
        assets.get(self.current.id()).unwrap().atlas.clone()
    }

    fn get_atlas_index(&self, assets: &Assets<ResolvedAnimationData>) -> usize {
        self.frame + self.facing as usize * assets.get(self.current.id()).unwrap().frames
    }
}

/// Maps character states to animation data
#[derive(Component, Debug, Clone, Reflect)]
pub struct AnimationStateMap(pub HashMap<TypeId, Handle<ResolvedAnimationData>>);

/// Resolved asset references for an animation, including handles to other assets
#[derive(Debug, Clone, PartialEq, Asset, Reflect)]
pub struct ResolvedAnimationData {
    pub image: Handle<Image>,
    pub atlas: TextureAtlas,
    pub frames: usize,
    pub interval: Duration,
}

fn resolve_animation_data(
    mut animation_registry: SystemRegistryMut<AnimationResource>,
    animation_sprite_registry: SystemRegistry<AnimationSpriteResource>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    let (animation_registry, animation_assets) = animation_registry.split();
    
    for animation in animation_registry.handles() {
        let animation = animation_assets.get_mut(&animation.clone())
            .unwrap_or_else(|| panic!("Failed to retrieve animation asset from handle from registry! This is a bug!"));

        let Some(image) = animation_sprite_registry.get_handle(animation.image.clone()) else {
            // TODO: Real error handling, since this could come up in normal operation
            dbg!("Failed to find image for animation: {:?}", animation.image.clone());
            return;
        };

        let layout = atlas_layouts.add(animation.atlas.clone());
        let atlas = TextureAtlas {
            layout,
            index: 0,
        };

        let resolved_animation = ResolvedAnimationData {
            image,
            atlas,
            frames: animation.frames,
            interval: animation.interval,
        };

        let resolved_animation_handle = asset_server.add(resolved_animation);
        animation.resolved_handle = Some(resolved_animation_handle);
    }
}

/// Stores the sprite and frame data for an animation
#[derive(Debug, Clone, PartialEq, Asset, TypePath)]
pub struct AnimationAsset {
    image: ResourceLocation<AnimationSpriteResource>,
    atlas: TextureAtlasLayout,
    frames: usize,
    interval: Duration,
    resolved_handle: Option<Handle<ResolvedAnimationData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AnimationCodec {
    pub format: u8,
    pub image: ResourceLocation<AnimationSpriteResource>,
    pub atlas: TextureAtlasCodec,
    pub frames: usize,
    pub duration: usize,
}
impl From<AnimationCodec> for AnimationAsset {
    fn from(codec: AnimationCodec) -> Self {
        AnimationAsset {
            image: codec.image,
            atlas: codec.atlas.into(),
            frames: codec.frames,
            interval: Duration::from_millis(codec.duration as u64),
            resolved_handle: None,
        }
    }
}

define_resource!(Animation, "animations", AnimationAsset, ResourceFileType::Data);
define_sprite_resource!(Animation, "animations");
