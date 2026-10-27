use crate::button::ButtonImpl;
use crate::theme::palette::{BackgroundInteractionPalette, SpriteInteractionPalette};
use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (
                apply_gamepad_sprite_interaction_palette,
                apply_sprite_interaction_palette,
            )
                .chain(),
            (
                apply_gamepad_background_interaction_palette,
                apply_background_interaction_palette,
            )
                .chain(),
        ),
    );

    app.add_observer(play_on_hover_sound_effect);
    app.add_observer(play_on_click_sound_effect);
}

fn apply_sprite_interaction_palette(
    mut palette_query: Query<
        (
            &Hovered,
            Option<&Pressed>,
            &SpriteInteractionPalette,
            &mut ImageNode,
        ),
        Or<(Changed<Hovered>, Changed<Pressed>)>,
    >,
    input_focus_visible: Res<InputFocusVisible>,
    mut commands: Commands,
) {
    for (hovered, pressed, palette, mut image) in &mut palette_query {
        let index = if pressed.is_some() {
            palette.pressed
        } else if hovered.get() {
            if input_focus_visible.0 {
                commands.insert_resource(InputFocusVisible(false));
            }
            palette.hovered
        } else {
            palette.none
        };

        image.texture_atlas.as_mut().unwrap().index = index;
    }
}

fn apply_gamepad_sprite_interaction_palette(
    input_focus: Res<InputFocus>,
    input_focus_visible: Res<InputFocusVisible>,
    mut palette_query: Query<(
        Entity,
        &Hovered,
        Option<&Pressed>,
        &SpriteInteractionPalette,
        &mut ImageNode,
    )>,
    button_query: Query<(Entity, &Children), With<ButtonImpl>>,
) {
    for (entity, hovered, pressed, palette, mut image) in palette_query.iter_mut() {
        let is_none = !hovered.get() && pressed.is_none();

        if input_focus_visible.0 {
            let focused_entity = input_focus.get();

            let focused = focused_entity == Some(entity)
                || button_query.iter().any(|(parent, children)| {
                    focused_entity == Some(parent) && children.contains(&entity)
                });

            if focused {
                image.texture_atlas.as_mut().unwrap().index = palette.hovered;
            } else {
                image.texture_atlas.as_mut().unwrap().index = palette.none;
            }
        } else if input_focus_visible.is_changed() && is_none {
            image.texture_atlas.as_mut().unwrap().index = palette.none;
        }
    }
}

fn apply_background_interaction_palette(
    mut palette_query: Query<
        (
            &Hovered,
            Option<&Pressed>,
            &BackgroundInteractionPalette,
            &mut BackgroundColor,
        ),
        Or<(Changed<Hovered>, Changed<Pressed>)>,
    >,
    input_focus_visible: Res<InputFocusVisible>,
    mut commands: Commands,
) {
    for (hovered, pressed, palette, mut color) in &mut palette_query {
        let new_color = if pressed.is_some() {
            palette.pressed
        } else if hovered.get() {
            if input_focus_visible.0 {
                commands.insert_resource(InputFocusVisible(false));
            }
            palette.hovered
        } else {
            palette.none
        };

        *color = BackgroundColor(new_color);
    }
}

fn apply_gamepad_background_interaction_palette(
    input_focus: Res<InputFocus>,
    input_focus_visible: Res<InputFocusVisible>,
    mut palette_query: Query<(
        Entity,
        &Hovered,
        Option<&Pressed>,
        &BackgroundInteractionPalette,
        &mut BackgroundColor,
    )>,
    button_query: Query<(Entity, &Children), With<ButtonImpl>>,
) {
    // For everything with a background color palette
    for (entity, hovered, pressed, palette, mut color) in palette_query.iter_mut() {
        let is_none = !hovered.get() && pressed.is_none();

        if input_focus_visible.0 {
            let focused_entity = input_focus.get();

            let focused = focused_entity == Some(entity)
                || button_query.iter().any(|(parent, children)| {
                    focused_entity == Some(parent) && children.contains(&entity)
                });

            if focused {
                *color = BackgroundColor(palette.hovered);
            } else {
                *color = BackgroundColor(palette.none);
            }
        } else if input_focus_visible.is_changed() && is_none {
            *color = BackgroundColor(palette.none);
        }
    }
}

// TODO: Update these assets to use resource locations
#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct InteractionAssets {
    #[dependency]
    pub hover: Handle<AudioSource>,
    #[dependency]
    pub click: Handle<AudioSource>,
}

impl FromWorld for InteractionAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            hover: assets.load("base/audio/sound_effects/button_hover.ogg"),
            click: assets.load("base/audio/sound_effects/button_click.ogg"),
        }
    }
}

fn play_on_hover_sound_effect(
    _trigger: On<PointerOver>,
    _commands: Commands,
    _interaction_assets: Option<Res<InteractionAssets>>,
    _interaction_query: Query<(), With<Hovered>>,
) {
    // TODO: Fix this after audio is separated into its own crate
    //command_window.spawn(sound_effect(interaction_assets.hover.clone()));
}

fn play_on_click_sound_effect(
    _trigger: On<PointerClick>,
    _commands: Commands,
    _interaction_assets: Option<Res<InteractionAssets>>,
    _interaction_query: Query<(), With<Hovered>>,
) {
    // TODO: Fix this after audio is separated into its own crate
    //command_window.spawn(sound_effect(interaction_assets.click.clone()));
}
