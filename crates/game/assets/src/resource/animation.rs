use crate::codec::{AnimationCodec, DistinctCodec, FixedIntervalCodec, FrameDataCodec};
use crate::loader::{LoaderJobManager, RonAssetLoader};
use crate::resource::characters::{CharacterSpriteRegistry, CharacterSpriteResource};
use bevy::prelude::*;
use data::prelude::*;
use data::resource::{ResourceVisitError, resource_kind};
use getset::{CloneGetters, Getters};
use std::collections::HashMap;
use std::time::Duration;
use tracing::info;

pub(in crate::resource) fn plugin(app: &mut App) {
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

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum FrameData {
    FixedInterval(FixedInterval),
    Distinct(Distinct),
}

impl FrameData {
    pub fn num_frames(&self) -> usize {
        match self {
            FrameData::FixedInterval(fixed_interval) => fixed_interval.indices.len(),
            FrameData::Distinct(distinct) => distinct.indices.len(),
        }
    }

    /// Get the duration of the given frame
    pub fn get_frame_duration(&self, frame: usize) -> Option<Duration> {
        match self {
            FrameData::FixedInterval(fixed_interval) => fixed_interval
                .indices
                .get(frame)
                .map(|_| fixed_interval.interval),
            FrameData::Distinct(distinct) => {
                distinct.indices.get(frame).map(|(_, interval)| *interval)
            }
        }
    }

    /// Get the duration of the given frame
    ///
    /// # Panics
    /// If the frame index is out of bounds
    pub fn frame_duration(&self, frame: usize) -> Duration {
        self.get_frame_duration(frame).expect("Frame index out of bounds")
    }

    /// Get the frame index of the given frame
    pub fn get_frame_index(&self, frame: usize) -> Option<usize> {
        match self {
            FrameData::FixedInterval(fixed_interval) => fixed_interval.indices.get(frame).copied(),
            FrameData::Distinct(distinct) => distinct.indices.get(frame).map(|(index, _)| *index),
        }
    }

    /// Get the frame index of the given frame
    ///
    /// # Panics
    /// If the frame index is out of bounds
    pub fn frame_index(&self, frame: usize) -> usize {
        self.get_frame_index(frame).expect("Frame index out of bounds")
    }
}

impl From<FrameDataCodec> for FrameData {
    fn from(value: FrameDataCodec) -> Self {
        match value {
            FrameDataCodec::FixedInterval(fixed_interval) => {
                FrameData::FixedInterval(fixed_interval.into())
            }
            FrameDataCodec::Distinct(distinct) => FrameData::Distinct(distinct.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct FixedInterval {
    interval: Duration,
    indices: Vec<usize>,
}
impl From<FixedIntervalCodec> for FixedInterval {
    fn from(value: FixedIntervalCodec) -> Self {
        FixedInterval {
            interval: Duration::from_millis(value.interval),
            indices: value.indices,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Distinct {
    indices: Vec<(usize, Duration)>,
}
impl From<DistinctCodec> for Distinct {
    fn from(value: DistinctCodec) -> Self {
        Distinct {
            indices: value
                .indices
                .into_iter()
                .map(|(k, v)| (k, Duration::from_millis(v)))
                .collect(),
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
