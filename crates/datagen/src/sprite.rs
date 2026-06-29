use game_core::datagen_api::*;

pub struct TextureAtlasData {
    size: (u32, u32),
    columns: u32,
    rows: u32,
    padding: Option<(u32, u32)>,
    offset: Option<(u32, u32)>,
}
impl TextureAtlasData {
    pub fn new(width: u32, height: u32, columns: u32, rows: u32) -> Self {
        Self {
            size: (width, height),
            columns,
            rows,
            padding: None,
            offset: None,
        }
    }
    
    pub fn with_padding(self, x: u32, y: u32) -> Self {
        Self {
            padding: Some((x, y)),
            ..self
        }
    }

    pub fn with_offset(self, x: u32, y: u32) -> Self {
        Self {
            offset: Some((x, y)),
            ..self
        }
    }
}
impl From<TextureAtlasData> for TextureAtlasCodec {
    fn from(value: TextureAtlasData) -> Self {
        Self {
            format: LATEST_FORMAT,
            size: value.size.into(),
            columns: value.columns,
            rows: value.rows,
            padding: value.padding.map(Into::into).into(),
            offset: value.offset.map(Into::into).into(),
        }
    }
}

const LATEST_FORMAT: u8 = 1;