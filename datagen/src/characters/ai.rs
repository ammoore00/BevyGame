use bevy_game_2d::data::resource::ResourceKind;
use bevy_game_2d::data::loc::ResourceLocation;
use bevy_game_2d::datagen_api::ai::{AiGraphCodec, AiGraphResource, AiStateCodec, AiStateResource};
use crate::{create_dir, write_data, WriteError};

pub(super) fn generate_generic_ai_data() -> Result<(), WriteError> {
    create_dir(AiStateResource::ROOT_DIR)?;
    create_dir(AiGraphResource::ROOT_DIR)?;

    write_data("idle".parse::<ResourceLocation<AiStateResource>>().unwrap(), &AiStateCodec::from(AiStateData {}))?;
    write_data("patrol".parse::<ResourceLocation<AiStateResource>>().unwrap(), &AiStateCodec::from(AiStateData {}))?;
    write_data("search".parse::<ResourceLocation<AiStateResource>>().unwrap(), &AiStateCodec::from(AiStateData {}))?;
    write_data("flee".parse::<ResourceLocation<AiStateResource>>().unwrap(), &AiStateCodec::from(AiStateData {}))?;
    write_data("move_to_target_loc".parse::<ResourceLocation<AiStateResource>>().unwrap(), &AiStateCodec::from(AiStateData {}))?;
    write_data("move_to_target_entity".parse::<ResourceLocation<AiStateResource>>().unwrap(), &AiStateCodec::from(AiStateData {}))?;
    write_data("attack_target".parse::<ResourceLocation<AiStateResource>>().unwrap(), &AiStateCodec::from(AiStateData {}))?;

    write_data("basic_wander".parse::<ResourceLocation<AiGraphResource>>().unwrap(), &AiGraphCodec::from(AiGraphData::new(
        "basic_wander",
        vec![
            "idle",
            "move_to_target_loc",
        ]
    )))?;

    Ok(())
}

#[derive(Debug, Clone)]
struct AiGraphData {
    loc: ResourceLocation<AiGraphResource>,
    states: Vec<ResourceLocation<AiStateResource>>,
}
impl AiGraphData {
    fn new(loc: &str, states: Vec<&str>) -> Self {
        Self {
            loc: loc.parse().unwrap(),
            states: states.into_iter().map(|state| state.parse().unwrap()).collect(),
        }
    }
}
impl From<AiGraphData> for AiGraphCodec {
    fn from(value: AiGraphData) -> Self {
        Self {
            format: LATEST_AI_GRAPH_FORMAT,
            states: value.states,
        }
    }
}

#[derive(Debug, Clone)]
struct AiStateData {
}
impl From<AiStateData> for AiStateCodec {
    fn from(_value: AiStateData) -> Self {
        AiStateCodec {
            format: LATEST_AI_STATE_FORMAT
        }
    }
}

const LATEST_AI_STATE_FORMAT: u8 = 1;
const LATEST_AI_GRAPH_FORMAT: u8 = 1;