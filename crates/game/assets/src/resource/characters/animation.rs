use crate::codec::{AnimationCodec, FrameDataCodec};
use crate::loader::{LoaderJobManager, RonAssetLoader};
use crate::resource::characters::{CharacterSpriteRegistry, CharacterSpriteResource};
use bevy::prelude::*;
use data::prelude::*;
use data::resource::{ResourceVisitError, resource_kind};
use getset::{CloneGetters, Getters};
use std::time::Duration;
use tracing::info;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<AnimationData>();
    app.init_asset_loader::<RonAssetLoader<AnimationCodec, AnimationData>>();
    app.add_registry_with_discovery::<AnimationResource>();
}

#[resource_kind(path = "characters/animations", asset_kind = AnimationData, visit_override = true)]
pub struct AnimationResource;
impl AnimationResource {
    fn visit(
        loc: ResourceLocation<Self>,
        animation: AnimationData,
        world: &mut World,
    ) -> Result<<Self as ResourceKind>::AssetKind, ResourceVisitError> {
        info!("Resolving animation: {}", loc);

        let animation = match animation {
            AnimationData::Resolved(_) => {
                error!("Visited animation which has already been resolved! {}", loc);
                return Err(ResourceVisitError(
                    "Animation has already been resolved".to_string(),
                ));
            }
            AnimationData::Partial(animation) => animation,
        };

        let sprite_registry = world.resource::<CharacterSpriteRegistry>();
        let Some(image) = sprite_registry.get(&animation.image).cloned() else {
            return Err(ResourceVisitError(format!(
                "Failed to find image for animation: {:?}",
                animation.image.clone()
            )));
        };

        let mut atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let layout = atlas_layouts.add(animation.atlas.clone());
        let atlas = TextureAtlas { layout, index: 0 };

        let frame_data = animation.frame_data.clone();

        let resolved_animation = ResolvedAnimationData {
            image,
            atlas,
            frame_data,
        };

        Ok(AnimationData::Resolved(resolved_animation))
    }
}

#[derive(Asset, Debug, Clone, PartialEq, TypePath)]
pub enum AnimationData {
    Resolved(ResolvedAnimationData),
    Partial(PartialAnimationData),
}
impl AnimationData {
    pub fn is_resolved(&self) -> bool {
        matches!(self, AnimationData::Resolved(_))
    }

    pub fn as_resolved(&self) -> Option<&ResolvedAnimationData> {
        match self {
            AnimationData::Resolved(data) => Some(data),
            AnimationData::Partial(_) => None,
        }
    }

    pub fn as_partial(&self) -> Option<&PartialAnimationData> {
        match self {
            AnimationData::Resolved(_) => None,
            AnimationData::Partial(data) => Some(data),
        }
    }

    pub fn unwrap(&self) -> &ResolvedAnimationData {
        self.as_resolved()
            .expect("Cannot unwrap unresolved animation data")
    }
}
impl From<AnimationCodec> for AnimationData {
    fn from(codec: AnimationCodec) -> Self {
        AnimationData::Partial(PartialAnimationData::from(codec))
    }
}

/// Resolved asset references for an animation, including handles to other resource
#[derive(Debug, Clone, PartialEq, Reflect, Getters, CloneGetters)]
pub struct ResolvedAnimationData {
    #[getset(get_clone = "pub")]
    image: Handle<Image>,
    #[getset(get = "pub")]
    atlas: TextureAtlas,
    #[getset(get = "pub")]
    frame_data: FrameData,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum FrameData {
    FixedInterval { frames: usize, interval: Duration },
    Distinct { intervals: Vec<Duration> },
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
            FrameDataCodec::FixedInterval {
                num_frames: frames,
                interval,
            } => FrameData::FixedInterval {
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
    resolved: bool,
}

impl From<AnimationCodec> for PartialAnimationData {
    fn from(codec: AnimationCodec) -> Self {
        PartialAnimationData {
            image: codec.image,
            atlas: codec.atlas.into(),
            frame_data: codec.frame_data.into(),
            resolved: false,
        }
    }
}
