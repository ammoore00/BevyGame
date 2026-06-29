use crate::codec::{AnimationCodec, FrameDataCodec};
use crate::game::character::animation::components::AnimationStateMap;
use crate::game::character::state::action_states::Attacking;
use crate::game::character::state::ActionStateTracker;
use crate::prelude::*;
use crate::{AssetLoadState};
use getset::{CloneGetters, Getters};
use std::time::Duration;
use bevy::ecs::system::SystemParam;
use tracing::{info, warn};

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<PartialAnimationData>();
    app.init_asset::<AnimationData>();
    app.init_asset_loader::<RonAssetLoader<AnimationCodec, PartialAnimationData>>();
    app.add_registry_with_discovery::<AnimationResource>();
    
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
    animation_context: &AnimationContext,
) -> Option<Handle<AnimationData>> {
    if let Some(attacking_state) = attacking_state {
        let Some(attack) = attack_context.get_asset(attacking_state.attack()) else {
            error!("Could not find attack definition for {}!", attacking_state.attack());
            return None;
        };

        // TODO: Improve error handling
        let Some(animation_handle) =
            animation_context.get_handle(attack.animation()).ok()
        else {
            error!("Could not find animation definition for attack: {}!", attacking_state.attack());
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
pub struct AnimationData {
    #[getset(get_clone = "pub")]
    image: Handle<Image>,
    #[getset(get = "pub")]
    atlas: TextureAtlas,
    #[getset(get = "pub")]
    frame_data: FrameData,
}

pub fn resolve_animation_data(
    mut resolved: Local<bool>,
    mut animation_registry: SystemRegistryMut<AnimationResource>,
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
            error!("Failed to find image for animation: {:?}", animation.image.clone());
            return;
        };

        let layout = atlas_layouts.add(animation.atlas.clone());
        let atlas = TextureAtlas {
            layout,
            index: 0,
        };

        let frame_data = animation.frame_data.clone();

        let resolved_animation = AnimationData {
            image,
            atlas,
            frame_data,
        };

        let resolved_animation_handle = asset_server.add(resolved_animation);
        animation.resolved_handle = Some(resolved_animation_handle);
        animation.resolved = true;
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
    resolved_handle: Option<Handle<AnimationData>>,
    resolved: bool,
}

impl From<AnimationCodec> for PartialAnimationData {
    fn from(codec: AnimationCodec) -> Self {
        PartialAnimationData {
            image: codec.image,
            atlas: codec.atlas.into(),
            frame_data: codec.frame_data.into(),
            resolved_handle: None,
            resolved: false,
        }
    }
}

define_data_resource!(Animation, "characters/animations", PartialAnimationData);

#[derive(SystemParam, Getters)]
pub struct AnimationContext<'w> {
    registry: SystemRegistry<'w, AnimationResource>,
    #[getset(get = "pub")]
    resolved_assets: Res<'w, Assets<AnimationData>>,
}
impl AnimationContext<'_> {
    pub fn get_handle(&self, loc: &ResourceLocation<AnimationResource>) -> Result<Handle<AnimationData>, AnimationContextError> {
        let partial_data = self.registry.get_asset(loc)
            .ok_or(AnimationContextError::NonexistentResourceLocation(loc.clone()))?;

        if !partial_data.resolved {
            return Err(AnimationContextError::NotYetResolved);
        }

        partial_data.resolved_handle.clone().ok_or(AnimationContextError::ResolvedAssetMissing)
    }
    
    pub fn get_data_from_handle(&self, handle: Handle<AnimationData>) -> Result<&AnimationData, AnimationContextError> {
        self.resolved_assets.get(&handle).ok_or(AnimationContextError::NonexistentResolvedAsset)
    }
    
    pub fn get_data(&self, loc: &ResourceLocation<AnimationResource>) -> Result<&AnimationData, AnimationContextError> {
        let handle = self.get_handle(loc)?;
        self.get_data_from_handle(handle)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AnimationContextError {
    #[error("Asset for resource location {0} does not exist!")]
    NonexistentResourceLocation(ResourceLocation<AnimationResource>),
    #[error("Resolved asset not yet loaded, please try again later")]
    NotYetResolved,
    #[error("Data does not have a resolved asset linked!")]
    ResolvedAssetMissing,
    #[error("Resolved asset does not exist!")]
    NonexistentResolvedAsset,
}