use std::collections::HashMap;
use crate::codec::sprite::TextureAtlasCodec;
use crate::resource::characters::CharacterSpriteResource;
use bevy::math::UVec2;
use bevy::prelude::TypePath;
use data::prelude::*;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixedIntervalCodec {
    pub interval: u64,
    pub indices: Vec<usize>,
}
impl FixedIntervalCodec {
    pub fn new(interval: u64, num_frames: usize) -> Self {
        Self {
            interval,
            indices: (0..num_frames).collect(),
        }
    }
    
    pub fn with_range(interval: u64, start_index: usize, num_frames: usize) -> Self {
        Self {
            interval,
            indices: (start_index..start_index + num_frames).collect(),
        }
    }
    
    pub fn with_indices(interval: u64, indices: Vec<usize>) -> Self {
        Self {
            interval,
            indices,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistinctCodec {
    pub indices: HashMap<usize, u64>,
}
impl DistinctCodec {
    pub fn new(intervals: Vec<u64>) -> Self {
        let mut indices = HashMap::new();
        for (index, interval) in intervals.into_iter().enumerate() {
            indices.insert(index, interval);
        }
        Self { indices }
    }
    
    pub fn with_range(intervals: Vec<u64>, start_index: usize, num_frames: usize) -> Self {
        let mut indices = HashMap::new();
        for index in start_index..start_index + num_frames {
            indices.insert(index, intervals[index - start_index]);
        }
        Self { indices }
    }
    
    pub fn with_indices(indices: HashMap<usize, u64>) -> Self {
        Self { indices }
    }
}
