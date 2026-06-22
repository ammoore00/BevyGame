use bevy::app::Update;
use bevy::color::Color;
use bevy::ecs::relationship::Relationship;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use getset::CopyGetters;
use std::ops::Not;
use tracing::info;
use crate::dev_tools::debug_menu::debug_options::{spawn_debug, DebugOptionState};
use crate::theme::widgets::text::{MEDIUM_FONT_SIZE, SMALL_FONT_SIZE, TINY_FONT_SIZE};

pub mod debug_options;
pub mod level_render;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        debug_options::plugin,
        level_render::plugin,
    ));

    app.init_resource::<DebugMenu>();

    app.add_systems(
        Update,
        toggle_debug_menu.run_if(input_just_pressed(TOGGLE_KEY))
    );

    app.add_systems(
        Update,
        (
            spawn_debug_menu,
            update_debug_menu,
            handle_debug_menu_buttons,
        ),
    );
}

pub const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

#[derive(Resource, Debug, Default)]
pub struct DebugMenu {
    open: bool,
}

#[derive(Component)]
struct DebugMenuRoot;

#[derive(EntityEvent, Debug, CopyGetters)]
pub struct DebugMenuEvent {
    #[getset(get_copy = "pub")]
    entity: Entity,
}

#[derive(Component, Debug, Clone)]
struct DebugCheckbox {
    enabled: bool,
}

pub trait DebugSetting: Component {}

#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct DebugButton;

#[derive(Component, Debug, Clone)]
struct DebugCategory {
    check_state: DebugCategoryCheckState,
    expanded: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) enum DebugCategoryCheckState {
    Empty,
    Partial,
    Checked,
}
impl DebugCategoryCheckState {
    fn icon(self) -> &'static str {
        match self {
            DebugCategoryCheckState::Empty => UNCHECKED,
            DebugCategoryCheckState::Partial => PARTIALLY_CHECKED,
            DebugCategoryCheckState::Checked => CHECKED,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        match (self, other) {
            (DebugCategoryCheckState::Checked, DebugCategoryCheckState::Checked) => DebugCategoryCheckState::Checked,
            (DebugCategoryCheckState::Empty, DebugCategoryCheckState::Empty) => DebugCategoryCheckState::Empty,
            _ => DebugCategoryCheckState::Partial,
        }
    }
}

impl From<bool> for DebugCategoryCheckState {
    fn from(value: bool) -> Self {
        if value {
            DebugCategoryCheckState::Checked
        } else {
            DebugCategoryCheckState::Empty
        }
    }
}
impl From<&[bool]> for DebugCategoryCheckState {
    fn from(values: &[bool]) -> Self {
        if values.iter().all(|&v| v) {
            DebugCategoryCheckState::Checked
        } else if values.iter().any(|&v| v) {
            DebugCategoryCheckState::Partial
        } else {
            DebugCategoryCheckState::Empty
        }
    }
}
impl From<DebugCategoryCheckState> for bool {
    fn from(value: DebugCategoryCheckState) -> Self {
        matches!(value, DebugCategoryCheckState::Checked)
    }
}

impl Not for DebugCategoryCheckState {
    type Output = DebugCategoryCheckState;

    fn not(self) -> Self::Output {
        match self {
            DebugCategoryCheckState::Empty => DebugCategoryCheckState::Checked,
            DebugCategoryCheckState::Partial => DebugCategoryCheckState::Checked,
            DebugCategoryCheckState::Checked => DebugCategoryCheckState::Empty,
        }
    }
}

#[derive(Component)]
struct DebugCategoryCheckbox;

#[derive(Component)]
struct DebugCategoryCollapseButton;

#[derive(Component)]
struct DebugCategoryContent;

const CHECKED: &str = "[x]";
const UNCHECKED: &str = "[ ]";
const PARTIALLY_CHECKED: &str = "[-]";
const COLLAPSED: &str = ">";
const EXPANDED: &str = "v";

pub fn toggle_debug_menu(mut debug_menu: ResMut<DebugMenu>) {
    debug_menu.open = !debug_menu.open;
    info!("Debug menu open: {}", debug_menu.open);
}

fn spawn_debug_menu(
    mut commands: Commands,
    debug_menu: Res<DebugMenu>,
    menu_query: Query<Entity, With<DebugMenuRoot>>,
    state: DebugOptionState,
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
                    width: Val::Px(400.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    row_gap: Val::Px(2.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.02, 0.02, 0.82)),
                GlobalZIndex(100),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Debug Menu"),
                    TextFont {
                        font_size: MEDIUM_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                spawn_debug(parent, state);
            });
    } else if !debug_menu.open && menu_exists {
        for entity in &menu_query {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn spawn_debug_category(
    parent: &mut ChildSpawnerCommands,
    label: impl Into<String>,
    checked: DebugCategoryCheckState,
    expanded: bool,
    spawn_contents: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                row_gap: Val::Px(2.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            DebugCategory {
                check_state: checked,
                expanded,
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(30.0),
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Percent(100.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                            DebugCategoryCollapseButton,
                            DebugButton,
                        ))
                        .with_children(|parent| {
                            let icon = if expanded { EXPANDED } else { COLLAPSED };
                            parent.spawn(text_bundle(icon));
                        });


                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                column_gap: Val::Px(8.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Start,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                            DebugCategoryCheckbox,
                            DebugButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn(text_bundle(checked.icon()));
                            parent.spawn(text_bundle(label.into().as_str()));
                        });

                });

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        margin: UiRect::left(Val::Px(40.0)),
                        flex_direction: FlexDirection::Column,
                        display: if expanded {
                            Display::Flex
                        } else {
                            Display::None
                        },
                        ..default()
                    },
                    DebugCategoryContent,
                ))
                .with_children(spawn_contents);
        });
}

fn text_bundle(icon: &str) -> impl Bundle {
    (
        Text::new(icon),
        TextFont {
            font_size: TINY_FONT_SIZE,
            ..default()
        },
        TextColor(Color::WHITE),
    )
}

pub(super) fn spawn_checkbox_row(
    parent: &mut ChildSpawnerCommands,
    label: impl Into<String>,
    checked: bool,
    marker: impl DebugSetting,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(95.0),
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
            DebugButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(if checked { CHECKED } else { UNCHECKED }),
                TextFont {
                    font_size: TINY_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: TINY_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn update_debug_menu(
    checkbox_query: Query<
        (&DebugCheckbox, &Children),
        Changed<DebugCheckbox>,
    >,
    mut category_query: Query<
        (&mut DebugCategory, &Children),
    >,
    category_checkbox_query: Query<&Children, With<DebugCategoryCheckbox>>,
    category_collapse_button_query: Query<&Children, With<DebugCategoryCollapseButton>>,
    category_content_query: Query<Entity, With<DebugCategoryContent>>,
    children_query: Query<&Children>,
    all_checkbox_query: Query<&DebugCheckbox>,
    mut node_query: Query<&mut Node>,
    mut text_query: Query<&mut Text>,
) {
    for (setting, children) in &checkbox_query {
        if let Some(checkbox_text_entity) = children.first()
            && let Ok(mut text) = text_query.get_mut(*checkbox_text_entity)
        {
            **text = if setting.enabled { CHECKED } else { UNCHECKED }.to_string();
        }
    }

    for (mut category, children) in &mut category_query {
        let check_state = debug_category_check_state(
            children,
            &children_query,
            &all_checkbox_query,
        );

        category.check_state = check_state;

        for child in children {
            update_debug_category_child_recursively(
                *child,
                &category,
                check_state,
                &children_query,
                &category_checkbox_query,
                &category_collapse_button_query,
                &category_content_query,
                &mut node_query,
                &mut text_query,
            );
        }
    }
}

fn debug_category_check_state(
    children: &Children,
    children_query: &Query<&Children>,
    checkbox_query: &Query<&DebugCheckbox>,
) -> DebugCategoryCheckState {
    let mut checked_count = 0;
    let mut total_count = 0;

    for child in children {
        let (child_checked_count, child_total_count) =
            count_debug_checkboxes_recursively(*child, children_query, checkbox_query);

        checked_count += child_checked_count;
        total_count += child_total_count;
    }

    match (checked_count, total_count) {
        (_, 0) | (0, _) => DebugCategoryCheckState::Empty,
        (checked_count, total_count) if checked_count == total_count => {
            DebugCategoryCheckState::Checked
        }
        _ => DebugCategoryCheckState::Partial,
    }
}

fn count_debug_checkboxes_recursively(
    entity: Entity,
    children_query: &Query<&Children>,
    checkbox_query: &Query<&DebugCheckbox>,
) -> (usize, usize) {
    let mut checked_count = 0;
    let mut total_count = 0;

    if let Ok(checkbox) = checkbox_query.get(entity) {
        total_count += 1;

        if checkbox.enabled {
            checked_count += 1;
        }
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children {
            let (child_checked_count, child_total_count) =
                count_debug_checkboxes_recursively(*child, children_query, checkbox_query);

            checked_count += child_checked_count;
            total_count += child_total_count;
        }
    }

    (checked_count, total_count)
}

fn update_debug_category_child_recursively(
    entity: Entity,
    category: &DebugCategory,
    check_state: DebugCategoryCheckState,
    children_query: &Query<&Children>,
    category_checkbox_query: &Query<&Children, With<DebugCategoryCheckbox>>,
    category_collapse_button_query: &Query<&Children, With<DebugCategoryCollapseButton>>,
    category_content_query: &Query<Entity, With<DebugCategoryContent>>,
    node_query: &mut Query<&mut Node>,
    text_query: &mut Query<&mut Text>,
) {
    if let Ok(button_children) = category_checkbox_query.get(entity)
        && let Some(text_entity) = button_children.first()
        && let Ok(mut text) = text_query.get_mut(*text_entity)
    {
        **text = check_state.icon().to_string();
    }

    if let Ok(button_children) = category_collapse_button_query.get(entity)
        && let Some(text_entity) = button_children.first()
        && let Ok(mut text) = text_query.get_mut(*text_entity)
    {
        **text = if category.expanded { EXPANDED } else { COLLAPSED }.to_string();
    }

    if category_content_query.get(entity).is_ok()
        && let Ok(mut node) = node_query.get_mut(entity)
    {
        node.display = if category.expanded {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children {
            update_debug_category_child_recursively(
                *child,
                category,
                check_state,
                children_query,
                category_checkbox_query,
                category_collapse_button_query,
                category_content_query,
                node_query,
                text_query,
            );
        }
    }
}

fn handle_debug_menu_buttons(
    mut checkbox_queries: ParamSet<(
        Query<
            (Entity, &Interaction, &mut BackgroundColor, Option<&mut DebugCheckbox>),
            (Changed<Interaction>, With<DebugButton>),
        >,
        Query<&mut DebugCheckbox>,
    )>,
    mut category_checkbox_query: Query<
        (&Interaction, &ChildOf),
        (Changed<Interaction>, With<DebugCategoryCheckbox>),
    >,
    mut category_collapse_query: Query<
        (&Interaction, &ChildOf),
        (Changed<Interaction>, With<DebugCategoryCollapseButton>),
    >,
    mut category_query: Query<(&mut DebugCategory, &Children)>,
    parent_query: Query<&ChildOf>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    for (entity, interaction, mut background_color, checkbox) in &mut checkbox_queries.p0() {
        match *interaction {
            Interaction::Pressed => {
                if let Some(mut setting) = checkbox {
                    setting.enabled = !setting.enabled;
                    commands.trigger(DebugMenuEvent {
                        entity,
                    });
                }

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

    let mut category_toggles = Vec::new();

    for (interaction, parent) in &mut category_checkbox_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let header_entity = parent.get();

        let Ok(category_parent) = parent_query.get(header_entity) else {
            continue;
        };

        category_toggles.push(category_parent.get());
    }

    for category_entity in category_toggles {
        if let Ok((mut category, children)) = category_query.get_mut(category_entity) {
            category.check_state = !category.check_state;
            let enabled = category.check_state;

            let mut checkbox_query = checkbox_queries.p1();

            for child in children {
                toggle_debug_checkboxes_recursively(
                    *child,
                    enabled.into(),
                    &children_query,
                    &mut checkbox_query,
                    &mut commands,
                );
            }
        }
    }

    for (interaction, parent) in &mut category_collapse_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let header_entity = parent.get();

        let Ok(category_parent) = parent_query.get(header_entity) else {
            continue;
        };

        let category_entity = category_parent.get();

        if let Ok((mut category, _)) = category_query.get_mut(category_entity) {
            category.expanded = !category.expanded;
        }
    }
}

fn toggle_debug_checkboxes_recursively(
    entity: Entity,
    enabled: bool,
    children_query: &Query<&Children>,
    checkbox_query: &mut Query<&mut DebugCheckbox>,
    commands: &mut Commands,
) {
    if let Ok(mut checkbox) = checkbox_query.get_mut(entity)
        && checkbox.enabled != enabled
    {
        checkbox.enabled = enabled;
        commands.trigger(DebugMenuEvent {
            entity,
        });
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children {
            toggle_debug_checkboxes_recursively(
                *child,
                enabled,
                children_query,
                checkbox_query,
                commands,
            );
        }
    }
}