use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::prelude::*;

use crate::theme::widget::ButtonRoot;
use crate::{asset_tracking::LoadResource, audio::sound_effect};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (apply_gamepad_interaction_palette, apply_interaction_palette).chain(),
    );

    app.load_resource::<InteractionAssets>();
    app.add_observer(play_on_hover_sound_effect);
    app.add_observer(play_on_click_sound_effect);
}

/// Palette for widget interactions. Add this to an entity that supports
/// [`Interaction`]s, such as a button, to change its [`BackgroundColor`] based
/// on the current interaction state.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InteractionPalette {
    pub none: usize,
    pub hovered: usize,
    pub pressed: usize,
}

fn apply_interaction_palette(
    mut palette_query: Query<
        (&Interaction, &InteractionPalette, &mut ImageNode),
        Changed<Interaction>,
    >,
    input_focus_visible: Res<InputFocusVisible>,
    mut commands: Commands,
) {
    // If there are any mouse interactions, disable the focus indicator
    let mut reset_focus = || {
        if input_focus_visible.0 {
            commands.insert_resource(InputFocusVisible(false));
        }
    };

    for (interaction, palette, mut image) in &mut palette_query {
        let index = match interaction {
            Interaction::None => palette.none,
            Interaction::Hovered => {
                reset_focus();
                palette.hovered
            }
            Interaction::Pressed => palette.pressed,
        };

        image.texture_atlas.as_mut().unwrap().index = index;
    }
}

fn apply_gamepad_interaction_palette(
    input_focus: Res<InputFocus>,
    input_focus_visible: Res<InputFocusVisible>,
    mut palette_query: Query<(Entity, &Interaction, &InteractionPalette, &mut ImageNode)>,
    button_query: Query<(Entity, &Children), With<ButtonRoot>>,
    interaction_assets: Option<Res<InteractionAssets>>,
    mut commands: Commands,
) {
    // For everything with a background color palette
    for (entity, interaction, palette, mut image) in palette_query.iter_mut() {
        // If we are rendering the current focused element
        if input_focus_visible.0 {
            // Iterate through each button that has children
            button_query.iter().for_each(|(parent, children)| {
                // If the button is the focused element
                if input_focus.0 == Some(parent) {
                    children.iter().for_each(|child| {
                        // If the entity we are currently looking at is a child of the focused button
                        if child == entity {
                            // Then update the color
                            image.texture_atlas.as_mut().unwrap().index = palette.hovered;
                        } else {
                            image.texture_atlas.as_mut().unwrap().index = palette.none;
                        }
                    })
                }
            });
        } else if input_focus_visible.is_changed() && matches!(interaction, Interaction::None) {
            // If the input is false, and has changed since the last frame, disable highlighting
            image.texture_atlas.as_mut().unwrap().index = palette.none;
        }
    }
}

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
            hover: assets.load("audio/sound_effects/button_hover.ogg"),
            click: assets.load("audio/sound_effects/button_click.ogg"),
        }
    }
}

fn play_on_hover_sound_effect(
    trigger: On<Pointer<Over>>,
    mut commands: Commands,
    interaction_assets: Option<Res<InteractionAssets>>,
    interaction_query: Query<(), With<Interaction>>,
) {
    let Some(interaction_assets) = interaction_assets else {
        return;
    };

    if interaction_query.contains(trigger.entity) {
        commands.spawn(sound_effect(interaction_assets.hover.clone()));
    }
}

fn play_on_click_sound_effect(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    interaction_assets: Option<Res<InteractionAssets>>,
    interaction_query: Query<(), With<Interaction>>,
) {
    let Some(interaction_assets) = interaction_assets else {
        return;
    };

    if interaction_query.contains(trigger.entity) {
        commands.spawn(sound_effect(interaction_assets.click.clone()));
    }
}