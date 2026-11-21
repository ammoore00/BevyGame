use crate::codec::sprite::TextureAtlasCodec;
use crate::resource::SpriteResource;
use bevy::math::UVec2;
use bevy::prelude::TypePath;
use data::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct AnimationCodec {
    pub format: u8,
    pub image: ResourceLocation<SpriteResource>,
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
            frame_data: FrameDataCodec::FixedInterval(FixedIntervalCodec::new(50, 8)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FrameDataCodec {
    FixedInterval(FixedIntervalCodec),
    Distinct(DistinctCodec),
}
impl FrameDataCodec {
    pub fn num_frames(&self) -> usize {
        match self {
            FrameDataCodec::FixedInterval(fixed) => fixed.indices.len(),
            FrameDataCodec::Distinct(distinct) => distinct.indices.len(),
        }
    }

    /// Shifts all indices by the given offset
    pub fn offset(&mut self, offset: usize) {
        match self {
            FrameDataCodec::FixedInterval(fixed) => {
                fixed.indices.iter_mut().for_each(|i| *i += offset)
            }
            FrameDataCodec::Distinct(distinct) => {
                distinct.indices.iter_mut().for_each(|i| i.0 += offset)
            }
        }
    }

    /// Clones the codec and shifts all indices by the given offset
    pub fn clone_with_offset(&self, offset: usize) -> Self {
        match self {
            FrameDataCodec::FixedInterval(fixed) => {
                FrameDataCodec::FixedInterval(FixedIntervalCodec {
                    interval: fixed.interval,
                    indices: fixed.indices.iter().map(|i| i + offset).collect(),
                })
            }
            FrameDataCodec::Distinct(distinct) => FrameDataCodec::Distinct(DistinctCodec {
                indices: distinct
                    .indices
                    .iter()
                    .map(|(i, _)| (i + offset, distinct.indices[*i].1))
                    .collect(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixedIntervalCodec {
    pub interval: u64,
    pub indices: Vec<usize>,
}
impl FixedIntervalCodec {
    pub fn new(num_frames: usize, interval: u64) -> Self {
        Self {
            interval,
            indices: (0..num_frames).collect(),
        }
    }

    pub fn with_range(start_index: usize, num_frames: usize, interval: u64) -> Self {
        Self {
            interval,
            indices: (start_index..start_index + num_frames).collect(),
        }
    }

    pub fn with_indices(indices: Vec<usize>, interval: u64) -> Self {
        Self { interval, indices }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistinctCodec {
    pub indices: Vec<(usize, u64)>,
}
impl DistinctCodec {
    pub fn new(intervals: Vec<u64>) -> Self {
        let mut indices = Vec::new();
        for (index, interval) in intervals.into_iter().enumerate() {
            indices.push((index, interval));
        }
        Self { indices }
    }

    pub fn with_range(start_index: usize, num_frames: usize, intervals: Vec<u64>) -> Self {
        let mut indices = Vec::new();
        for index in start_index..start_index + num_frames {
            indices.push((index, intervals[index - start_index]));
        }
        Self { indices }
    }

    pub fn with_indices(indices: Vec<(usize, u64)>) -> Self {
        Self { indices }
    }
}
