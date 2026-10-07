use bevy::prelude::*;
use widgets::text::{TextFormatting, SMALL_FONT_SIZE};

pub struct DebugLabel {
    pub name: String,
    pub name_color: Color,

    pub text: String,
    pub text_color: Color,
}
impl DebugLabel {
    pub fn as_scene(&self) -> impl Scene {
        let name_formatting = TextFormatting::new(SMALL_FONT_SIZE, self.name_color);
        let text_formatting = TextFormatting::new(SMALL_FONT_SIZE, self.text_color);

        bsn! [
            Node {
                flex_direction: FlexDirection::Row,
            }
            Children [
                widgets::text::text(self.name.as_str(), name_formatting),
                widgets::text::text(self.text.as_str(), text_formatting),
            ]
        ]
    }
}
