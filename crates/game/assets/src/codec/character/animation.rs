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
            frame_data: FrameDataCodec::FixedInterval {
                num_frames: 8,
                interval: 50,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FrameDataCodec {
    FixedInterval { num_frames: usize, interval: u64 },
    Distinct { intervals: Vec<u64> },
}

impl FrameDataCodec {
    pub fn num_frames(&self) -> u32 {
        match self {
            FrameDataCodec::FixedInterval { num_frames, .. } => *num_frames as u32,
            FrameDataCodec::Distinct { intervals } => intervals.len() as u32,
        }
    }
}
