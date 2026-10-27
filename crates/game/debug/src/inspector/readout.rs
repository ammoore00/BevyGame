use bevy::prelude::*;
use getset::Getters;
use widgets::text::{TextFormatting, SMALL_FONT_SIZE, TINY_FONT_SIZE, MEDIUM_FONT_SIZE};
use widgets::theme::palette::PRIMARY_TEXT;
use crate::inspector::window::{data_node, InspectorDataReadoutMarker};

const INDENT_WIDTH: u32 = 16;

const TITLE_FONT_SIZE: FontSize = MEDIUM_FONT_SIZE;
const SECTION_FONT_SIZE: FontSize = SMALL_FONT_SIZE;
const ITEM_FONT_SIZE: FontSize = TINY_FONT_SIZE;

#[derive(Debug, Clone, Getters)]
pub struct InspectorReadout {
    #[getset(get = "pub")]
    title: String,
    #[getset(get = "pub")]
    sections: Vec<ReadoutSection>,
}
impl InspectorReadout {
    pub fn new(title: impl AsRef<str>) -> Self {
        Self {
            title: title.as_ref().to_string(),
            sections: Vec::new(),
        }
    }
    
    pub fn with_title(mut self, title: impl AsRef<str>) -> Self {
        self.title = title.as_ref().to_string();
        self
    }
    
    pub fn section(mut self, section: ReadoutSection) -> Self {
        self.sections.push(section);
        self
    }
    
    pub fn _push_sections(mut self, sections: Vec<ReadoutSection>) -> Self {
        self.sections.extend(sections);
        self
    }
    
    pub fn _extend(&mut self, other: Self) {
        self.sections.extend(other.sections);
    }
    
    pub fn with_extension(mut self, other: Self) -> Self {
        self.sections.extend(other.sections);
        self
    }
    
    pub fn as_scene(&self) -> impl Scene {
        let sections = self
            .sections
            .iter()
            .map(|section| section.as_scene())
            .collect::<Vec<_>>();

        bsn! {
            InspectorDataReadoutMarker
            @data_node()
            Children [
                @data_node()
                @widgets::text::text(
                    self.title.as_str(),
                    TextFormatting {
                        font_size: TITLE_FONT_SIZE,
                        color: PRIMARY_TEXT,
                    }
                )
                TextLayout {
                    justify: Justify::Start
                }
                --
                {sections}
            ]
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReadoutSection {
    title: String,
    items: Vec<ReadoutItem>,
}
impl ReadoutSection {
    pub fn new(title: impl AsRef<str>) -> Self {
        Self {
            title: title.as_ref().to_string(),
            items: Vec::new(),
        }
    }
    
    pub fn subsection(mut self, subsection: Self) -> Self {
        self.items.push(ReadoutItem::Subsection(subsection));
        self
    }
    
    pub fn item(mut self, item: impl AsRef<str>) -> Self {
        self.items.push(ReadoutItem::Item(item.as_ref().to_string()));
        self
    }
    
    pub fn maybe_item(mut self, item: Option<impl AsRef<str>>) -> Self {
        if let Some(item) = item {
            self.items.push(ReadoutItem::Item(item.as_ref().to_string()));
        }
        self
    }
    
    pub fn as_scene(&self) -> Box<dyn Scene> {
        if self.items.is_empty() {
            return Box::new(bsn! {});
        }
        
        let items = self
            .items
            .iter()
            .map(|item| item.as_scene())
            .collect::<Vec<_>>();
        
        Box::new(bsn! {
            @data_node()
            Children [
                @data_node()
                @widgets::text::text(
                    self.title.as_str(),
                    TextFormatting {
                        font_size: SECTION_FONT_SIZE,
                        color: PRIMARY_TEXT,
                    }
                )
                TextLayout {
                    justify: Justify::Start
                }
                --
                @data_node()
                Node {
                    padding: UiRect::left(px(INDENT_WIDTH))
                }
                Children [
                    {items}
                ]
            ]
        })
    }
}

#[derive(Debug, Clone)]
pub enum ReadoutItem {
    Subsection(ReadoutSection),
    Item(String),
}
impl ReadoutItem {
    pub fn as_scene(&self) -> Box<dyn Scene> {
        match self {
            ReadoutItem::Subsection(subsection) => Box::new(bsn! {
                @{subsection.as_scene()}
            }) as Box<dyn Scene>,
            ReadoutItem::Item(text) => Box::new(bsn! {
                @data_node()
                @widgets::text::text(
                    text,
                    TextFormatting {
                        font_size: ITEM_FONT_SIZE,
                        color: PRIMARY_TEXT,
                    }
                )
                TextLayout {
                    justify: Justify::Start
                }
            }) as Box<dyn Scene>,
        }
    }
}
