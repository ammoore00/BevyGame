use crate::button::scene::ButtonImpl;
use crate::theme::palette::SpriteInteractionPalette;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_button_style);
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub enum ButtonStyle {
    #[default]
    Default,
    ArrowRight,
    //ArrowLeft,
    //ArrowUp,
    ArrowDown,
    //Plus,
    //Minus,
    Back,
}

impl ButtonStyle {
    const ROWS: u32 = 8;
    const COLS: u32 = 8;

    pub(crate) fn make_slicer(&self) -> TextureSlicer {
        TextureSlicer {
            border: BorderRect::all(4.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 16.0,
        }
    }

    // TODO: Cache these layouts
    pub(crate) fn make_layout(&self) -> TextureAtlasLayout {
        TextureAtlasLayout::from_grid(UVec2::splat(16), Self::COLS, Self::ROWS, None, None)
    }

    pub(crate) fn get_indices(&self) -> (usize, usize, usize) {
        match self {
            ButtonStyle::Default => (Self::idx(0, 0), Self::idx(0, 1), Self::idx(0, 2)),
            ButtonStyle::ArrowRight => (Self::idx(1, 0), Self::idx(1, 1), Self::idx(1, 2)),
            ButtonStyle::ArrowDown => (Self::idx(4, 0), Self::idx(4, 1), Self::idx(4, 2)),
            ButtonStyle::Back => (Self::idx(7, 0), Self::idx(7, 1), Self::idx(7, 2)),
        }
    }

    pub(crate) fn make_palette_scene(self) -> impl Scene {
        let indices = self.get_indices();
        bsn![SpriteInteractionPalette {
            none: { indices.0 },
            hovered: { indices.1 },
            pressed: { indices.2 },
        }]
    }

    pub(crate) fn get_palette(&self) -> SpriteInteractionPalette {
        let indices = self.get_indices();
        SpriteInteractionPalette {
            none: { indices.0 },
            hovered: { indices.1 },
            pressed: { indices.2 },
        }
    }

    fn idx(row: u32, col: u32) -> usize {
        (row * Self::COLS + col) as usize
    }

    pub(crate) fn get_index(&self) -> usize {
        self.get_indices().0
    }
}

/// Detects changes to the button style and applies the appropriate visual state.
fn update_button_style(
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut button_query: Query<
        (
            &ButtonStyle,
            &Interaction,
            &mut ImageNode,
            &mut SpriteInteractionPalette,
        ),
        (With<ButtonImpl>, Changed<ButtonStyle>),
    >,
) {
    for (style, interaction, mut image_node, mut interaction_palette) in &mut button_query {
        apply_button_style(
            *style,
            *interaction,
            &mut texture_atlas_layouts,
            &mut image_node,
            &mut interaction_palette,
        );
    }
}

fn apply_button_style(
    style: ButtonStyle,
    interaction: Interaction,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    image_node: &mut ImageNode,
    interaction_palette: &mut SpriteInteractionPalette,
) {
    let layout = texture_atlas_layouts.add(style.make_layout());
    let palette = style.get_palette();

    let index = match interaction {
        Interaction::None => palette.none,
        Interaction::Hovered => palette.hovered,
        Interaction::Pressed => palette.pressed,
    };

    image_node.image_mode = NodeImageMode::Sliced(style.make_slicer());
    image_node.texture_atlas = Some(TextureAtlas { layout, index });

    *interaction_palette = palette;
}
