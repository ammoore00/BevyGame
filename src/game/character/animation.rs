use crate::data::loader::{LoaderJobManager, RonAssetLoader};
use crate::data::registry::{ResolvedResourceRegistry, ResolvedSystemRegistry, ResolvedSystemRegistryMut, SystemRegistry};
use crate::data::sprite::TextureAtlasCodec;
use crate::data::resource::ResourceFileType;
use crate::datagen_api::assets::CharacterSpriteResource;
use crate::datagen_api::attack::AttackResource;
use crate::game::character::state::action_states::Attacking;
use crate::game::character::state::ActionStateTracker;
use crate::game::character::Facing;
use crate::screens::Screen;
use crate::{define_resolvable_resource, AssetLoadState};
use crate::{AppSystems, AssetSystems, PausableSystems};
use bevy::prelude::*;
use getset::{CloneGetters, Getters};
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
use crate::data::loc::ResourceLocation;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<PartialAnimationData>();
    app.init_asset::<ResolvedAnimationData>();
    app.init_asset_loader::<RonAssetLoader<AnimationCodec, PartialAnimationData>>();
    app.add_resolved_registry_with_discovery::<AnimationResource>();

    app.add_systems(
        OnEnter(AssetLoadState::Resolving),
        resolve_animation_data.in_set(AssetSystems::ResolveAssets)
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
    mut query: Query<(
        &ActionStateTracker,
        &Facing,
        &AnimationStateMap,
        &mut CharacterAnimationTracker,
        Option<&Attacking>,
    )>,
    attack_context: SystemRegistry<AttackResource>,
    animation_context: ResolvedSystemRegistry<AnimationResource>,
) {
    for (
        state_tracker,
        facing,
        animation_state_map,
        mut animation_tracker,
        attacking_state
    ) in &mut query {
        animation_tracker.facing = *facing;

        let Some(animation_handle) = get_animation_handle(
            state_tracker,
            animation_state_map,
            attacking_state,
            &attack_context,
            &animation_context,
        ) else {
            warn!("Failed to get animation handle!");
            return;
        };

        // Update animation tracker state if the animation has changed
        animation_tracker.current_animation = animation_handle.clone();
        let animation = animation_context.get_resolved_asset_from_handle(animation_handle.clone()).unwrap();

        let interval = animation.frame_data.frame_duration(0).unwrap();

        if animation_tracker.prev_animation != animation_tracker.current_animation {
            animation_tracker.timer = Timer::new(interval, TimerMode::Repeating);
            animation_tracker.frame = 0;
        }

        animation_tracker.prev_animation = animation_handle;
    }
}

fn update_animation_atlas(
    mut query: Query<(
        &ActionStateTracker,
        &CharacterAnimationTracker,
        &AnimationStateMap,
        &mut Sprite,
        Option<&Attacking>,
    )>,
    attack_context: SystemRegistry<AttackResource>,
    animation_context: ResolvedSystemRegistry<AnimationResource>,
) {
    for (
        state_tracker,
        animation_tracker,
        animation_state_map,
        mut sprite,
        attacking_state
    ) in &mut query {
        let Some(animation_handle) = get_animation_handle(
            state_tracker,
            animation_state_map,
            attacking_state,
            &attack_context,
            &animation_context,
        ) else {
            warn!("Failed to get animation handle!");
            return;
        };

        let animation = animation_context.get_resolved_asset_from_handle(animation_handle).unwrap();

        sprite.image = animation.image.clone();

        let mut atlas = animation.atlas.clone();
        // Calculate index: (Direction Row * Frames per row) + Current Frame
        atlas.index = animation_tracker.get_atlas_index(animation_context.resolved_assets());
        sprite.texture_atlas = Some(atlas);
    }
}

fn get_animation_handle(
    state_tracker: &ActionStateTracker,
    animation_state_map: &AnimationStateMap,
    attacking_state: Option<&Attacking>,
    attack_context: &SystemRegistry<AttackResource>,
    animation_context: &ResolvedSystemRegistry<AnimationResource>,
) -> Option<Handle<ResolvedAnimationData>> {
    if let Some(attacking_state) = attacking_state {
        let Some(attack) = attack_context.get_asset(attacking_state.attack()) else {
            warn!("Could not find attack definition for {}!", attacking_state.attack());
            return None;
        };

        let Some(animation_handle) = animation_context.get_resolved_handle(attack.animation()) else {
            warn!("Could not find animation definition for attack: {}!", attacking_state.attack());
            return None;
        };

        Some(animation_handle)
    } else if let Some(animation_handle) = animation_state_map.0.get(&state_tracker.type_id).cloned() {
        Some(animation_handle)
    } else {
        warn!("Could not find animation data for state!");
        None
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct CharacterAnimationTracker {
    default_animation: Handle<ResolvedAnimationData>,
    current_animation: Handle<ResolvedAnimationData>,
    prev_animation: Handle<ResolvedAnimationData>,

    facing: Facing,
    timer: Timer,
    frame: usize,
}

impl CharacterAnimationTracker {
    pub fn new(
        default: Handle<ResolvedAnimationData>,
        assets: &Assets<ResolvedAnimationData>,
    ) -> Self {
        let frame_data = assets.get(default.id()).unwrap().frame_data();
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

        let animation = assets.get(self.current_animation.id()).unwrap();

        self.frame = (self.frame + 1) % animation.frame_data().num_frames();
        self.timer.set_duration(animation.frame_data().frame_duration(self.frame).unwrap());
    }

    fn get_image(&self, assets: &Assets<ResolvedAnimationData>) -> Handle<Image> {
        assets.get(self.current_animation.id()).unwrap().image().clone()
    }

    fn get_atlas(&self, assets: &Assets<ResolvedAnimationData>) -> TextureAtlas {
        assets.get(self.current_animation.id()).unwrap().atlas().clone()
    }

    fn get_atlas_index(&self, assets: &Assets<ResolvedAnimationData>) -> usize {
        self.frame + self.facing as usize * assets.get(self.current_animation.id()).unwrap().frame_data().num_frames()
    }
}

/// Maps character states to animation data
#[derive(Component, Debug, Clone, Reflect)]
pub struct AnimationStateMap(pub HashMap<TypeId, Handle<ResolvedAnimationData>>);
impl AnimationStateMap {
    pub fn from_resource_location_map(map: &HashMap<TypeId, ResourceLocation<AnimationResource>>, registry: &ResolvedResourceRegistry<AnimationResource>) -> Self {
        let resolved_map = map.iter()
            .map(|(type_id, location)| {
                (*type_id, registry.get(location).unwrap().clone())
            })
            .collect();
        AnimationStateMap(resolved_map)
    }
}

/// Resolved asset references for an animation, including handles to other assets
#[derive(Debug, Clone, PartialEq, Asset, Reflect, Getters, CloneGetters)]
pub struct ResolvedAnimationData {
    #[getset(get_clone = "pub")]
    image: Handle<Image>,
    #[getset(get = "pub")]
    atlas: TextureAtlas,
    #[getset(get = "pub")]
    frame_data: FrameData,
}

fn resolve_animation_data(
    mut resolved: Local<bool>,
    mut animation_registry: ResolvedSystemRegistryMut<AnimationResource>,
    animation_sprite_registry: SystemRegistry<CharacterSpriteResource>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    if *resolved {
        return;
    }
    
    info!("RESOLVING ANIMATIONS");

    let (
        animation_registry,
        resolved_animation_registry,
        animation_assets,
    ) = animation_registry.split();

    for (loc, animation) in animation_registry.iter() {
        info!("Resolving animation: {}", loc);

        let animation = &mut animation_assets.get_mut(&animation.clone())
            .unwrap_or_else(|| {
                panic!(
                    "Failed to retrieve animation asset from registry! This is a bug!\n\
                    Expected Resource: {}\n\
                    Expected Path: {}",
                    loc, loc.as_path().to_string_lossy()
                )
            });

        let Some(image) = animation_sprite_registry.get_handle(&animation.image) else {
            // TODO: Real error handling, since this could come up in normal operation
            warn!("Failed to find image for animation: {:?}", animation.image.clone());
            return;
        };

        let layout = atlas_layouts.add(animation.atlas.clone());
        let atlas = TextureAtlas {
            layout,
            index: 0,
        };

        let frame_data = animation.frame_data.clone();

        let resolved_animation = ResolvedAnimationData {
            image,
            atlas,
            frame_data,
        };

        let resolved_animation_handle = asset_server.add(resolved_animation);
        resolved_animation_registry.register_asset(loc.clone(), resolved_animation_handle.clone());
        animation.resolved_handle = Some(resolved_animation_handle);
    }

    *resolved = true;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum FrameData {
    FixedInterval {
        frames: usize,
        interval: Duration,
    },
    Distinct {
        intervals: Vec<Duration>,
    }
}
impl FrameData {
    pub fn num_frames(&self) -> usize {
        match self {
            FrameData::FixedInterval { frames, .. } => *frames,
            FrameData::Distinct { intervals } => intervals.len(),
        }
    }

    pub fn frame_duration(&self, frame: usize) -> Option<Duration> {
        match self {
            FrameData::FixedInterval { interval, .. } => Some(*interval),
            FrameData::Distinct { intervals } => intervals.get(frame).copied(),
        }
    }
}
impl From<FrameDataCodec> for FrameData {
    fn from(value: FrameDataCodec) -> Self {
        match value {
            FrameDataCodec::FixedInterval { num_frames: frames, interval } => FrameData::FixedInterval {
                frames,
                interval: Duration::from_millis(interval),
            },
            FrameDataCodec::Distinct { intervals } => FrameData::Distinct {
                intervals: intervals.into_iter().map(Duration::from_millis).collect(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FrameDataCodec {
    FixedInterval {
        num_frames: usize,
        interval: u64,
    },
    Distinct {
        intervals: Vec<u64>,
    }
}
impl FrameDataCodec {
    pub fn num_frames(&self) -> u32 {
        match self {
            FrameDataCodec::FixedInterval { num_frames, .. } => *num_frames as u32,
            FrameDataCodec::Distinct { intervals } => intervals.len() as u32,
        }
    }
}

/// Stores the sprite and frame data for an animation
#[derive(Debug, Clone, PartialEq, Asset, TypePath)]
pub struct PartialAnimationData {
    image: ResourceLocation<CharacterSpriteResource>,
    atlas: TextureAtlasLayout,
    frame_data: FrameData,
    resolved_handle: Option<Handle<ResolvedAnimationData>>,
}
impl From<AnimationCodec> for PartialAnimationData {
    fn from(codec: AnimationCodec) -> Self {
        PartialAnimationData {
            image: codec.image,
            atlas: codec.atlas.into(),
            frame_data: codec.frame_data.into(),
            resolved_handle: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AnimationCodec {
    pub format: u8,
    pub image: ResourceLocation<CharacterSpriteResource>,
    pub atlas: TextureAtlasCodec,
    pub frame_data: FrameDataCodec,
}
impl AnimationCodec {
    pub const LATEST_FORMAT: u8 = 1;
}
impl Default for AnimationCodec {
    fn default() -> Self {
        Self {
            format: Self::LATEST_FORMAT,
            image: "untitled".parse().unwrap(),
            atlas: TextureAtlasCodec {
                format: TextureAtlasCodec::LATEST_FORMAT,
                size: UVec2::splat(64),
                columns: 8,
                rows: 8,
                padding: Default::default(),
                offset: Default::default(),
            },
            frame_data: FrameDataCodec::FixedInterval {
                num_frames: 8,
                interval: 50,
            },
        }
    }
}

define_resolvable_resource!(Animation, "characters/animations", PartialAnimationData, ResolvedAnimationData);
