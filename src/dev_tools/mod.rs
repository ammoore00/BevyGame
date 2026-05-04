//! Development tools for the game. This plugin is only enabled in dev builds.

use std::fmt::Debug;
use bevy::{
    dev_tools::states::log_transitions,
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    app.init_resource::<DebugMenu>();

    // Toggle the debug menu.
    app.add_systems(
        Update,
        toggle_debug_menu.run_if(input_just_pressed(TOGGLE_KEY)),
    );

    app.add_systems(
        Update,
        (
            spawn_debug_menu,
            update_debug_menu,
            handle_debug_menu_buttons,
        ),
    );

    app.add_observer(on_ui_debug);
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

#[derive(Resource, Debug, Default)]
struct DebugMenu {
    open: bool,
}

#[derive(Component)]
struct DebugMenuRoot;

#[derive(EntityEvent, Debug)]
struct DebugMenuEvent {
    entity: Entity,
}

#[derive(Component, Debug, Clone)]
struct DebugCheckbox {
    enabled: bool,
}

trait DebugSetting: Component {}

#[derive(Component, Debug, Clone)]
struct DebugUi;
impl DebugSetting for DebugUi {}

fn toggle_debug_menu(mut debug_menu: ResMut<DebugMenu>) {
    debug_menu.open = !debug_menu.open;
    info!("Debug menu open: {}", debug_menu.open);
}

fn spawn_debug_menu(
    mut commands: Commands,
    debug_menu: Res<DebugMenu>,
    menu_query: Query<Entity, With<DebugMenuRoot>>,
    ui_debug_options: Res<UiDebugOptions>,
) {
    if !debug_menu.is_changed() {
        return;
    }

    let menu_exists = !menu_query.is_empty();

    if debug_menu.open && !menu_exists {
        commands
            .spawn((
                Name::new("Debug Menu"),
                DebugMenuRoot,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(16.0),
                    left: Val::Px(16.0),
                    width: Val::Px(320.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    row_gap: Val::Px(8.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.02, 0.02, 0.82)),
                ZIndex(1000),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Debug Menu"),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                spawn_checkbox_row(
                    parent,
                    "UI debug overlay",
                    ui_debug_options.enabled,
                    DebugUi,
                );
            });
    } else if !debug_menu.open && menu_exists {
        for entity in &menu_query {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_checkbox_row(
    parent: &mut ChildSpawnerCommands,
    label: impl Into<String>,
    checked: bool,
    marker: impl DebugSetting,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(28.0),
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
            DebugCheckbox {
                enabled: checked,
            },
            marker,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(if checked { CHECKED } else { UNCHECKED }),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn update_debug_menu(
    checkbox_query: Query<
        (&DebugCheckbox, &Children),
        (Changed<DebugCheckbox>)
    >,
    mut text_query: Query<&mut Text>,
) {
    for (setting, children) in &checkbox_query {
        if let Some(checkbox_text_entity) = children.first()
            && let Ok(mut text) = text_query.get_mut(*checkbox_text_entity)
        {
            **text = if setting.enabled { CHECKED } else { UNCHECKED }.to_string();
        }
    }
}

const CHECKED: &str = "[x]";
const UNCHECKED: &str = "[ ]";

fn handle_debug_menu_buttons(
    mut interaction_query: Query<
        (Entity, &Interaction, &mut BackgroundColor, &mut DebugCheckbox),
        (Changed<Interaction>),
    >,
    mut commands: Commands,
) {
    for (entity, interaction, mut background_color, mut setting) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                setting.enabled = !setting.enabled;
                commands.trigger(DebugMenuEvent {
                    entity,
                });
                *background_color = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.20));
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0));
            }
        }
    }
}

macro_rules! debug_menu_event {
    (
        $marker:ty,
        fn $fn_name:ident(
            $event:ident: $event_ty:ty,
            $($args:tt)*
        ) $content:block
    ) => {
        fn $fn_name(
            $event: $event_ty,
            __entity_query: Query<Entity, With<$marker>>,
            $($args)*
        ) {
            let $event: On<DebugMenuEvent> = $event;
            match __entity_query.single() {
                Ok(entity) => {
                    if $event.entity == entity {
                        $content
                    }
                }
                Err(err) => {
                    error!("Failed to obtain entity: {}", err);
                }
            }
        }
    };
}

debug_menu_event!(
    DebugUi,
    fn on_ui_debug(
        event: On<DebugMenuEvent>,
        mut ui_debug_options: ResMut<UiDebugOptions>,
    ) {
        ui_debug_options.toggle();
        info!("UI Debug toggled: {}", ui_debug_options.enabled);
    }
);