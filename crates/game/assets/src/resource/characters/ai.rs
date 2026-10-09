use crate::codec::{AiCodec, AiHearingCodec, AiSensesCodec, AiSightCodec};
use crate::loader::{LoaderJobManager, RonAssetLoader};
use bevy::prelude::*;
use data::prelude::resource_kind;

pub(super) fn plugin(app: &mut App) {
    app.add_registry_with_discovery::<AiResource>();
    app.init_asset::<AiParams>();
    app.init_asset_loader::<RonAssetLoader<AiCodec, AiParams>>();
}

#[resource_kind(path = "characters/ai", asset_kind = AiParams)]
pub struct AiResource;

macro_rules! optional_scene {
    ($option:expr) => {
        match $option {
            Some(value) => Box::new(bsn! { value }) as Box<dyn Scene>,
            None => Box::new(bsn! {}) as Box<dyn Scene>,
        }
    };
}

macro_rules! optional_components_scene {
    ($($option:expr),* $(,)?) => {
        bsn! {
            $( @{ optional_scene!($option) } )*
        }
    };
}

#[derive(Asset, Debug, Clone, TypePath)]
pub struct AiParams {
    senses: Option<AiSenseParams>,
    behavior: AiBehaviorParams,
}
impl AiParams {
    pub fn as_scene(&self) -> impl Scene {
        let senses = match self.senses {
            Some(ref senses) => Box::new(bsn! { @{senses.as_scene()} }) as Box<dyn Scene>,
            None => Box::new(bsn! {}) as Box<dyn Scene>,
        };
        let behavior = self.behavior.clone();
        
        bsn! {
            @senses
            behavior
        }
    }
}
impl From<AiCodec> for AiParams {
    fn from(value: AiCodec) -> Self {
        let behavior = value.clone().into();
        
        Self {
            senses: value.senses.map(Into::into),
            behavior,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiSenseParams {
    sight: Option<AiSightParams>,
    hearing: Option<AiHearingParams>,
}
impl AiSenseParams {
    pub fn as_scene(&self) -> impl Scene {
        optional_components_scene! {
            self.sight.clone(),
            self.hearing.clone(),
        }
    }
}
impl Default for AiSenseParams {
    fn default() -> Self {
        Self {
            sight: Some(AiSightParams::default()),
            hearing: Some(AiHearingParams::default()),
        }
    }
}
impl From<AiSensesCodec> for AiSenseParams {
    fn from(value: AiSensesCodec) -> Self {
        Self {
            sight: value.sight.into_option().map(Into::into),
            hearing: value.hearing.into_option().map(Into::into),
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct AiSightParams {
    /// Range in meters of the sight cone
    range: f32,
    /// Angle in degrees from the center of the sight cone to the edge
    half_angle: f32,
}
impl AiSightParams {
    const DEFAULT: AiSightParams = Self {
        range: 5.0,
        half_angle: 45.0,
    };
}
impl Default for AiSightParams {
    fn default() -> Self {
        Self::DEFAULT
    }
}
impl From<AiSightCodec> for AiSightParams {
    fn from(value: AiSightCodec) -> Self {
        Self {
            range: value.range.unwrap_or(Self::DEFAULT.range),
            half_angle: value.half_angle.unwrap_or(Self::DEFAULT.half_angle),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct AiHearingParams {}
impl Default for AiHearingParams {
    fn default() -> Self {
        Self {}
    }
}
impl From<AiHearingCodec> for AiHearingParams {
    fn from(_value: AiHearingCodec) -> Self {
        Self {}
    }
}

const DEFAULT_LEASH_DISTANCE: f32 = 10.0;

#[derive(Component, Debug, Clone)]
pub struct AiBehaviorParams {
    leash_distance: Option<f32>,
}
impl AiBehaviorParams {
    const DEFAULT: Self = Self {
        leash_distance: Some(DEFAULT_LEASH_DISTANCE),
    };
}
impl Default for AiBehaviorParams {
    fn default() -> Self {
        Self::DEFAULT
    }
}
impl From<AiCodec> for AiBehaviorParams {
    fn from(value: AiCodec) -> Self {
        Self {
            leash_distance: value.leash_distance.into(),
        }
    }
}
