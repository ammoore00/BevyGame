use crate::codec::{AnimationCodec, FrameDataCodec};
use crate::data::registry::{ResolvedSystemRegistry, ResolvedSystemRegistryMut};
use crate::game::character::animation::components::AnimationStateMap;
use crate::game::character::state::action_states::Attacking;
use crate::game::character::state::ActionStateTracker;
use crate::prelude::*;
use crate::{define_resolvable_resource, AssetLoadState};
use getset::{CloneGetters, Getters};
use std::time::Duration;
use tracing::{info, warn};

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<PartialAnimationData>();
    app.init_asset::<ResolvedAnimationData>();
    app.init_asset_loader::<RonAssetLoader<AnimationCodec, PartialAnimationData>>();
    app.add_resolved_registry_with_discovery::<AnimationResource>();
    
    app.add_systems(
        OnEnter(AssetLoadState::Resolving),
        resolve_animation_data.in_set(AssetSystems::ResolveAssets)
    );
}

pub fn get_animation_handle(
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

pub fn resolve_animation_data(
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

define_resolvable_resource!(Animation, "characters/animations", PartialAnimationData, ResolvedAnimationData);