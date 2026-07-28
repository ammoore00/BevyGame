use crate::theme::palette::TRANSPARENT_OVERLAY;
use assets::resource::UiSpriteResource;
use bevy::ecs::template::OptionTemplate;
use bevy::image::TextureAtlasTemplate;
use bevy::prelude::*;
use data::prelude::loc;

pub fn ui_root() -> impl Scene {
    bsn! [
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
        }
        Pickable::IGNORE
    ]
}

pub fn scrollable_ui_root() -> impl Scene {
    bsn! [
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            overflow: Overflow {
                x: OverflowAxis::Visible,
                y: OverflowAxis::Scroll,
            },
        }
        Pickable::IGNORE
    ]
}

pub fn ui_background(style: UiBackgroundStyle) -> Box<dyn Scene> {
    if let Ok(style) = UiBackgroundImage::try_from(style) {
        Box::new(bsn![
            #UiBackground
            ImageNode {
                image: {loc::<UiSpriteResource>("background").unwrap()},
                image_mode: NodeImageMode::Sliced({style.make_slicer()}),
                texture_atlas: OptionTemplate::Some(TextureAtlasTemplate {
                    layout: asset_value(style.make_layout()),
                    index: {style.get_index()}
                }),
            }
        ]) as Box<dyn Scene>
    } else {
        Box::new(bsn![
            #UiBackground
            BackgroundColor(TRANSPARENT_OVERLAY)
        ]) as Box<dyn Scene>
    }
}

pub enum UiBackgroundStyle {
    Main,
    Panel,
    Transparent,
}
impl TryFrom<UiBackgroundStyle> for UiBackgroundImage {
    type Error = String;

    fn try_from(value: UiBackgroundStyle) -> std::result::Result<Self, Self::Error> {
        match value {
            UiBackgroundStyle::Main => Ok(UiBackgroundImage::Main),
            UiBackgroundStyle::Panel => Ok(UiBackgroundImage::Panel),
            UiBackgroundStyle::Transparent => Err(
                "Transparent background style cannot be converted to UiBackgroundImage".to_string(),
            ),
        }
    }
}

pub enum UiBackgroundImage {
    Main,
    Panel,
}
impl UiBackgroundImage {
    fn make_slicer(&self) -> TextureSlicer {
        match self {
            UiBackgroundImage::Main => TextureSlicer {
                border: BorderRect::all(8.0),
                center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
                max_corner_scale: 2.0,
            },
            UiBackgroundImage::Panel => TextureSlicer {
                border: BorderRect::all(4.0),
                center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
                max_corner_scale: 2.0,
            },
        }
    }

    fn make_layout(&self) -> TextureAtlasLayout {
        match self {
            UiBackgroundImage::Main => {
                TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 4, None, None)
            }
            UiBackgroundImage::Panel => TextureAtlasLayout::from_grid(
                UVec2::splat(24),
                4,
                4,
                Some(UVec2::splat(8)),
                Some(UVec2::splat(4)),
            ),
        }
    }

    fn get_index(&self) -> usize {
        match self {
            UiBackgroundImage::Main => 0,
            UiBackgroundImage::Panel => 1,
        }
    }
}
