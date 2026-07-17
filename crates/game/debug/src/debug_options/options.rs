use crate::debug_options::window::DebugWindowInitialized;
use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use common::dev_tools::*;
use common::marker;
use std::marker::PhantomData;
use widgets::button::ButtonWithTextOptions;
use widgets::text::{MEDIUM_FONT_SIZE, SMALL_FONT_SIZE, TINY_FONT_SIZE};
use widgets::theme::palette::{BackgroundInteractionPalette, BUTTON_TEXT};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<NavMapNodesRes>();
    app.init_resource::<NavMapEdgesRes>();

    app.init_resource::<CharacterCollisionRes>();
    app.init_resource::<TileCollisionRes>();

    app.init_resource::<UiRenderRes>();
    
    app.add_observer(on_debug_window_initialized);
}

fn on_debug_window_initialized(
    _: On<DebugWindowInitialized>,
    debug_buttons: Query<Entity, With<DebugButton>>,
    mut commands: Commands,
) {
    for button in debug_buttons.iter() {
        commands.trigger(CheckDebugState(button));
    }
}

pub(super) fn global_debug() -> impl Scene {
    bsn! [
        #GlobalDebug
        widgets::background::ui_root()
        Node {
            position_type: PositionType::Relative,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            row_gap: px(8)
        }
        Children [
            (
                widgets::text::text("Debug Options", MEDIUM_FONT_SIZE, BUTTON_TEXT)
                Node {
                    justify_self: JustifySelf::Start,
                }
            ),
            navigation(),
            physics(),
            ui(),
        ]
    ]
}

fn debug_category(display: &str) -> impl Scene {
    bsn! [
        DebugCategory
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,

            width: percent(100),
        }
        Children [
            (
                Node
                Children [
                    (
                        widgets::text::text(display, SMALL_FONT_SIZE, BUTTON_TEXT)
                        Node {
                            justify_self: JustifySelf::Start,
                        }
                    ),
                ]
            )
        ]
    ]
}

macro_rules! debug_option_list {
    ($($option:expr),* $(,)?) => {
        bsn! [
            Children [
                Node {
                    padding: {UiRect::left(px(24)).with_right(px(16))},
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Start,
                }
                Children [
                    $(
                        {$option},
                    )*
                ]
            ]
        ]
    };
}

macro_rules! debug_option {
    ($option:ident, $display:literal) => {
        bsn! [
            @DebugEntry<$option> {
                @display: $display
            }
            Node
        ]
    };
}

marker!(DebugCategory);
marker!(DebugButton);

#[derive(SceneComponent, Debug, Clone, Copy, Default)]
#[scene(DebugButtonProps)]
struct DebugEntry<T>(PhantomData<T>)
where T: DebugOption<Mutability = Mutable> + Unpin;

impl<T> DebugEntry<T>
where T: DebugOption<Mutability = Mutable> + Unpin
{
    fn scene(props: DebugButtonProps) -> impl Scene {
        bsn! [
            T
            DebugEntry<T>
            Node
            Children [
                debug_option_button::<T>(props.display)
            ]
        ]
    }
}

struct DebugButtonProps {
    display: &'static str,
}
impl Default for DebugButtonProps {
    fn default() -> Self {
        Self {
            display: "<unnamed debug option>",
        }
    }
}

const TRANSPARENT_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
const HOVERED_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.2);
const PRESSED_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);

const CHECKED: &str = "[x]";
const UNCHECKED: &str = "[ ]";

const SEPARATOR: char = '-';

fn debug_option_button<T: DebugOption<Mutability = Mutable>>(text: impl AsRef<str>) -> impl Scene {
    let text = format!("{} {} {}", UNCHECKED, SEPARATOR, text.as_ref());

    let options = ButtonWithTextOptions {
        font_size: TINY_FONT_SIZE,
        width: percent(100),
        height: Val::Auto,
        justify_content: JustifyContent::Start,
        ..default()
    };

    let palette = BackgroundInteractionPalette {
        none: TRANSPARENT_COLOR,
        hovered: HOVERED_COLOR,
        pressed: PRESSED_COLOR,
    };

    let button = widgets::button::with_text_inline(
        text,
        options,
        palette,
        on_debug_option_button_pressed::<T>
    );

    bsn! [
        button
        DebugButton
        Node {
            justify_self: JustifySelf::Start,
            padding: UiRect::left(px(4)),
        }
        on(on_debug_option_button_spawned::<T>)
    ]
}

#[derive(EntityEvent)]
struct CheckDebugState(Entity);

fn on_debug_option_button_spawned<T: DebugOption<Mutability = Mutable>>(
    event: On<CheckDebugState>,
    button_query: Query<
        (Entity, &ChildOf, &Children),
        With<DebugButton>,
    >,
    text_query: Query<(Entity, &mut Text)>,
    ui_state_query: Query<&mut T, With<Children>>,
    debug_state: Res<T::Res>,
) {
    let new_state = if debug_state.get() {
        DebugUiState::True
    } else {
        DebugUiState::False
    };

    let result = set_debug_ui_state::<T>(event.0, button_query, text_query, ui_state_query, new_state);

    if let Err(err) = result {
        error!("{}", err);
    }
}

fn on_debug_option_button_pressed<T: DebugOption<Mutability = Mutable>>(
    event: On<Pointer<Click>>,
    button_query: Query<
        (Entity, &ChildOf, &Children),
        With<DebugButton>,
    >,
    text_query: Query<(Entity, &mut Text)>,
    ui_state_query: Query<&mut T, With<Children>>,
    debug_state: ResMut<T::Res>,
) {
    let result = set_debug_ui_state::<T>(event.entity, button_query, text_query, ui_state_query, DebugUiState::Toggle);

    match result {
        Ok(new_state) => {
            debug_state.into_inner().set(new_state);
        },
        Err(err) => {
            error!("{}", err);
        }
    }
}

fn set_debug_ui_state<T: DebugOption<Mutability = Mutable>>(
    entity: Entity,
    mut button_query: Query<
        (Entity, &ChildOf, &Children),
        With<DebugButton>,
    >,
    mut text_query: Query<(Entity, &mut Text)>,
    mut ui_state_query: Query<&mut T, With<Children>>,
    new_state: DebugUiState,
) -> Result<bool> {
    let Ok((_, button_parent, button_children)) = button_query.get_mut(entity) else {
        return Err("Failed to get button from event".into());
    };

    let Some(mut button_text) = text_query.iter_mut()
        .find(|(entity, _)| button_children.contains(entity))
        .map(|(_, text)| text)
    else {
        return Err("Failed to get button text from button children".into());
    };

    let Ok(mut ui_state) = ui_state_query.get_mut(button_parent.0) else {
        return Err("Failed to get ui state from button parent".into());
    };

    let new_state = match new_state {
        DebugUiState::True => true,
        DebugUiState::False => false,
        DebugUiState::Toggle => !ui_state.get(),
    };

    info!("Setting debug state {:?} to {}", ui_state, new_state);

    ui_state.set(new_state);

    let marker = if new_state { CHECKED } else { UNCHECKED };

    // Unwrap is safe because we know there will always be a space
    // separating the marker from the label
    let split = button_text.0.split_once(SEPARATOR).unwrap();
    button_text.0 = format!("{} {}{}", marker, SEPARATOR, split.1);

    Ok(new_state)
}

enum DebugUiState {
    True,
    False,
    Toggle,
}

//------ Global Debug ------//

// Navigation

fn navigation() -> impl Scene {
    bsn! [
        #NavigationDebug
        debug_category("Navigation")
        {debug_option_list!(
            debug_option!(NavMapNodes, "Render Navigation Nodes"),
            debug_option!(NavMapEdges, "Render Navigation Edges"),
        )}
    ]
}

#[derive(Component, Default, Clone, Debug, DebugOption, Reflect)]
#[reflect(Component)]
pub struct NavMapNodes(bool);

#[derive(Component, Default, Clone, Debug, DebugOption, Reflect)]
#[reflect(Component)]
pub struct NavMapEdges(bool);

// Physics

fn physics() -> impl Scene {
    bsn! [
        #PhysicsDebug
        debug_category("Physics")
        {debug_option_list!(
            debug_option!(CharacterCollision, "Render Character Collision"),
            debug_option!(TileCollision, "Render Tile Collision"),
        )}
    ]
}

#[derive(Component, Default, Clone, Debug, DebugOption, Reflect)]
#[reflect(Component)]
pub struct CharacterCollision(bool);

#[derive(Component, Default, Clone, Debug, DebugOption, Reflect)]
#[reflect(Component)]
pub struct TileCollision(bool);

// User Interface

fn ui() -> impl Scene {
    bsn! [
        #UiDebug
        debug_category("User Interface")
        {debug_option_list!(
            debug_option!(UiRender, "Render User Interface Debug"),
        )}
    ]
}

#[derive(Component, Default, Clone, Debug, DebugOption, Reflect)]
#[reflect(Component)]
pub struct UiRender(bool);

//------ Per-Entity Debug ------//

// Navigation

#[derive(Component, Default, Clone, Debug, DebugOption, Reflect)]
#[reflect(Component)]
pub struct NpcPath(bool);