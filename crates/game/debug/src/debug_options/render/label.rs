use bevy::prelude::*;
use widgets::text::TextFormatting;

pub struct DebugLabel {
    pub name: String,
    pub name_formatting: TextFormatting,
    
    pub text: String,
    pub text_formatting: TextFormatting,
}