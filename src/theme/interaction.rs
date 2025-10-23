use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, audio::sound_effect};

pub(super) fn plugin(app: &mut App) {
    //app.add_systems(Update, apply_interaction_palette);
    app.add_systems(Update, apply_gamepad_interaction_palette);

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
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn apply_interaction_palette(
    mut palette_query: Query<
        (&Interaction, &InteractionPalette, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut input_focus_visible: ResMut<InputFocusVisible>,
) {
    for (interaction, palette, mut background) in &mut palette_query {
        *background = match interaction {
            Interaction::None => palette.none,
            Interaction::Hovered => {
                //input_focus_visible.set(Box::new(false)).expect("Failed to set InputFocusVisible");
                palette.hovered
            },
            Interaction::Pressed => palette.pressed,
        }
        .into();
    }
}

fn apply_gamepad_interaction_palette(
    input_focus: Res<InputFocus>,
    input_focus_visible: Res<InputFocusVisible>,
    mut palette_query: Query<
        (Entity, &InteractionPalette, &mut BackgroundColor),
    >,
) {
    for (entity, palette, mut background) in palette_query.iter_mut() {
        if input_focus.0 == Some(entity) {//&& input_focus_visible.0 {
            println!("Focused element is {entity}");
            *background = palette.hovered.into();
        } else {
            *background = palette.none.into();
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct InteractionAssets {
    #[dependency]
    hover: Handle<AudioSource>,
    #[dependency]
    click: Handle<AudioSource>,
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
