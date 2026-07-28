use crate::dev_tools::editor::file_manager::{
    EditorResourceKind, FileKind, FileManager, FileTaskChannelSet,
};
use crate::screens::Screen;
use assets::resource::characters::{AnimationResource, AttackResource, CharacterResource};
use bevy::ecs::query::QuerySingleError;
use bevy::ecs::relationship::Relationship;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::marker;
use data::loc::AnyResourceLocation;
use data::prelude::*;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;
use widgets::button::{ButtonStyle, ButtonWithTextOptions};
use widgets::text::{MEDIUM_FONT_SIZE, SMALL_FONT_SIZE};
use widgets::theme::palette::{BackgroundInteractionPalette, HEADER_TEXT};
use widgets::{button, text};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (
                (
                    spawn_collapsible_menu_contents::<CharacterMenu>,
                    update_menu_contents_from_registry::<CharacterMenu>,
                )
                    .chain(),
                (
                    spawn_collapsible_menu_contents::<AnimationMenu>,
                    update_menu_contents_from_registry::<AnimationMenu>,
                )
                    .chain(),
                (
                    spawn_collapsible_menu_contents::<AttackMenu>,
                    update_menu_contents_from_registry::<AttackMenu>,
                )
                    .chain(),
            ),
            update_menu_items,
            finalize_menu_items,
            render_menu_items,
        )
            .chain()
            .run_if(in_state(Screen::Editor)),
    );
}

marker!(FileBrowser);

pub(super) fn spawn_file_browser() -> impl Scene {
    bsn! [
        #FileBrowser
        FileBrowser
        widgets::background::scrollable_ui_root()
        Node {
            position_type: PositionType::Relative,
            justify_content: JustifyContent::FlexStart,
            row_gap: px(2.),
        }
        Children [
            (
                #CharacterMenu
                CharacterMenu
                collapsible_menu("Characters", MEDIUM_FONT_SIZE)
            ),
            (
                #AnimationMenu
                AnimationMenu
                collapsible_menu("Animations", MEDIUM_FONT_SIZE)
            ),
            (
                #AttackMenu
                AttackMenu
                collapsible_menu("Attacks", MEDIUM_FONT_SIZE)
            ),
        ]
    ]
}

#[derive(Component, Debug, Clone, Copy)]
struct Collapsed(bool);
impl Default for Collapsed {
    fn default() -> Self {
        Self(true)
    }
}

fn collapsible_menu(text: impl Into<String>, font_size: impl Into<FontSize>) -> impl Scene {
    bsn! [
        Collapsed
        Node {
            flex_direction: FlexDirection::Column,
            width: percent(100),
        }
        Children [
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(8),
                width: percent(100),
            }
            Children [
                button::with_style(ButtonStyle::ArrowRight, 2, {
                    |
                        event: On<Pointer<Click>>,
                        parent_query: Query<&ChildOf>,
                        mut commands: Commands,
                        mut menu_query: Query<&mut Collapsed>,
                    | {
                        let Ok(menu_inner) = parent_query.get(event.entity).map(ChildOf::get) else {
                            error!("Failed to get menu inner");
                            return;
                        };

                        let Ok(menu) = parent_query.get(menu_inner).map(ChildOf::get) else {
                            error!("Failed to get menu root");
                            return;
                        };

                        if let Ok(mut collapsed) = menu_query.get_mut(menu) {
                            collapsed.0 = !collapsed.0;

                            let style = if collapsed.0 {
                                ButtonStyle::ArrowRight
                            } else {
                                ButtonStyle::ArrowDown
                            };

                            commands.entity(event.entity).insert(style);
                        } else {
                            error!("Failed to get collapsed component for menu");
                        }
                    }
                }),
                text::text(text, font_size, HEADER_TEXT)
            ]
        ]
    ]
}

trait MenuContentsKind: Component + Debug {
    type ResourceKind: EditorResourceKind;
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct CharacterMenu;
impl MenuContentsKind for CharacterMenu {
    type ResourceKind = CharacterResource;
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AnimationMenu;
impl MenuContentsKind for AnimationMenu {
    type ResourceKind = AnimationResource;
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AttackMenu;
impl MenuContentsKind for AttackMenu {
    type ResourceKind = AttackResource;
}

#[derive(Component, Debug)]
struct MenuContentsUntypedMarker;

#[derive(Component, Debug)]
struct MenuContents<T: MenuContentsKind> {
    _phantom_data: PhantomData<T>,
}
impl<T: MenuContentsKind> MenuContents<T> {
    fn new() -> Self {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

#[derive(Component, Debug, Default)]
struct MenuContentsUninitialized;
#[derive(Component, Debug, Default)]
struct MenuContentsProcessing;
#[derive(Component, Debug, Default)]
struct MenuContentsFinalized;

const CONTENT_START_PADDING: f32 = 30.;
const CONTENT_INNER_PADDING: f32 = 20.;
const CONTENT_PADDING: f32 = 1.;

/// Check the collapsed state of the menu entity and spawn or despawn the content entity as needed
fn spawn_collapsible_menu_contents<ContentKind>(
    menu_query: Query<(Entity, &Collapsed), With<ContentKind>>,
    contents_query: Query<Entity, With<MenuContents<ContentKind>>>,
    mut commands: Commands,
) where
    ContentKind: MenuContentsKind,
{
    let Ok((menu, collapsed)) = menu_query.single() else {
        error!("Failed to get collapsed component for menu");
        return;
    };

    match contents_query.single() {
        Ok(content_entity) => {
            if collapsed.0 {
                commands.entity(content_entity).despawn()
            }
        }
        Err(QuerySingleError::NoEntities(_)) => {
            if !collapsed.0 {
                let contents = commands
                    .spawn((
                        MenuContentsUntypedMarker,
                        MenuContents::<ContentKind>::new(),
                        MenuContentsUninitialized,
                        Node {
                            width: percent(100),
                            padding: UiRect::px(
                                CONTENT_START_PADDING,
                                CONTENT_PADDING,
                                CONTENT_PADDING,
                                CONTENT_PADDING,
                            ),

                            ..Default::default()
                        },
                    ))
                    .id();
                commands.entity(menu).add_child(contents);
            }
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            error!("Multiple entities found for menu contents")
        }
    }
}

trait MenuRegistryAccessor<T: ResourceKind>: SystemParam {
    type AssetKind: Asset;
    fn iter(&self) -> impl Iterator<Item = (ResourceLocation<T>, Handle<Self::AssetKind>)>;
}

impl<'w, T: ResourceKind> MenuRegistryAccessor<T> for SystemRegistry<'w, T> {
    type AssetKind = T::AssetKind;
    fn iter(&self) -> impl Iterator<Item = (ResourceLocation<T>, Handle<Self::AssetKind>)> {
        self.registry()
            .iter()
            .map(|(location, handle)| (location.clone(), handle.clone()))
    }
}

fn update_menu_contents_from_registry<ContentKind: MenuContentsKind>(
    contents_query: Query<
        (Entity, Option<&MenuContentsUninitialized>),
        With<MenuContents<ContentKind>>,
    >,
    registry: SystemRegistry<ContentKind::ResourceKind>,
    commands: Commands,
) {
    update_menu_contents_inner::<ContentKind, _, _>(contents_query, registry, commands)
}

/// Find all resource locations to populate the given menu
/// and adds them to the list in a hierarchical folder structure
fn update_menu_contents_inner<ContentKind, Registry, Resource>(
    contents_query: Query<
        (Entity, Option<&MenuContentsUninitialized>),
        With<MenuContents<ContentKind>>,
    >,
    registry: Registry,
    mut commands: Commands,
) where
    ContentKind: MenuContentsKind,
    Registry: MenuRegistryAccessor<Resource>,
    Resource: EditorResourceKind,
{
    // TODO: Figure out a way to make this work with change detection?

    let single = contents_query.single();
    let (contents, uninitialized) = match single {
        Ok(single) => single,
        Err(QuerySingleError::NoEntities(_)) => {
            // If there isn't a matching entity, that's fine,
            // as that just means that the menu isn't open
            return;
        }
        Err(QuerySingleError::MultipleEntities(err)) => {
            error!("Failed to get menu contents entity: {}", err);
            return;
        }
    };

    let file_kind = FileKind::from_resource_kind::<Resource>();

    if uninitialized.is_some() {
        for (loc, _) in registry.iter() {
            let item = commands
                .spawn(UninitializedMenuItem(
                    loc.as_local_path().to_path_buf(),
                    loc.into(),
                    file_kind,
                ))
                .id();
            commands.entity(contents).add_child(item);
        }
    }
}

#[derive(Component, Debug, Clone)]
struct UninitializedMenuItem(PathBuf, AnyResourceLocation, FileKind);

#[derive(Component, Debug, Clone)]
enum MenuItem {
    Folder(String),
    File(String, AnyResourceLocation, FileKind),
}
impl MenuItem {
    fn name(&self) -> &str {
        match self {
            MenuItem::Folder(name) => name,
            MenuItem::File(name, _, _) => name,
        }
    }
}

/// Process menu items marked as uninitialized and populate their children
fn update_menu_items(
    // Query for the item itself which we want to update
    items_query: Query<(Entity, &UninitializedMenuItem, &ChildOf)>,
    // Query for getting the parent of the item
    parent_query: Query<
        (Entity, &Children, Option<&MenuContentsUninitialized>),
        Or<(With<MenuItem>, With<MenuContentsUntypedMarker>)>,
    >,
    // Query for getting the siblings of the item
    sibling_query: Query<(Entity, &MenuItem)>,
    mut commands: Commands,
) {
    // Only process one item per frame in order to prevent duplicate items
    // TODO: Fix this to make it not take so long
    for (item_entity, item, parent) in items_query.iter().take(1) {
        let path = item.0.clone();
        let loc = item.1.clone();
        let file_kind = item.2;

        let mut components = path.components().peekable();
        let Some(component) = components.next() else {
            error!("Cannot create empty path");
            commands.entity(item_entity).despawn();
            continue;
        };

        let component = component.as_os_str().to_string_lossy();
        let component = component.as_ref();

        let remaining_path = path.strip_prefix(component).unwrap().to_path_buf();

        let Ok((parent_entity, siblings, menu_uninitialized)) = parent_query.get(parent.0) else {
            error!("Cannot find parent for menu item");
            commands.entity(item_entity).despawn();
            continue;
        };

        if menu_uninitialized.is_some() {
            commands
                .entity(parent_entity)
                .remove::<MenuContentsUninitialized>();
            commands
                .entity(parent_entity)
                .insert(MenuContentsProcessing);
        }

        let mut siblings = sibling_query
            .iter()
            .filter(|(sibling_entity, _)| siblings.contains(sibling_entity));

        // Check if the current component is the last one, and thus we are at the final file
        let is_last = components.peek().is_none();

        // See if an item already exists for this component
        let existing = siblings.find(|(_, sibling_item)| match (sibling_item, is_last) {
            (MenuItem::Folder(name), false) | (MenuItem::File(name, _, _), true) => {
                component == name
            }
            _ => false,
        });

        match (is_last, existing) {
            //If we are on the actual file:
            // If it already exists, despawn the uninitialized item
            (true, Some(_)) => {
                commands.entity(item_entity).despawn();
            }
            // Otherwise, replace the uninitialized reference with an item
            (true, None) => {
                commands
                    .entity(item_entity)
                    .remove::<UninitializedMenuItem>();
                commands.entity(item_entity).insert(MenuItem::File(
                    component.to_string(),
                    loc,
                    file_kind,
                ));
            }
            //If we are still in a directory:
            // If the child already exists, despawn the uninitialized entity,
            // then add a new uninitialized one under the existing match
            (false, Some((existing, _))) => {
                commands.entity(item_entity).despawn();
                let child = commands
                    .spawn(UninitializedMenuItem(remaining_path, loc, file_kind))
                    .id();
                commands.entity(existing).add_child(child);
            }
            // Otherwise, replace the uninitialized reference with a folder,
            // then add an uninitialized child entity
            (false, None) => {
                commands
                    .entity(item_entity)
                    .remove::<UninitializedMenuItem>();
                commands
                    .entity(item_entity)
                    .insert(MenuItem::Folder(component.to_string()));
                let child = commands
                    .spawn(UninitializedMenuItem(remaining_path, loc, file_kind))
                    .id();
                commands.entity(item_entity).add_child(child);
            }
        }
    }
}

/// Once all processing is done, finalize the menu items
fn finalize_menu_items(
    // Find if there are any items marked as uninitialized
    uninitialized_query: Query<(), With<UninitializedMenuItem>>,
    // Query children of menu tree roots
    menu_query: Query<Entity, With<MenuContentsProcessing>>,
    mut commands: Commands,
) {
    // Wait until all components are initialized
    if !uninitialized_query.is_empty() {
        return;
    }

    for menu in menu_query {
        commands.entity(menu).remove::<MenuContentsProcessing>();
        commands.entity(menu).insert(MenuContentsFinalized);
    }
}

const TRANSPARENT_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
const HOVERED_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.2);
const PRESSED_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);

const BROWSER_BUTTON_PADDING: f32 = 6.;
const BROWSER_BUTTON_MARGIN: f32 = 12.;

fn render_menu_items(
    // Only process rendering information if everything is done processing
    processing_query: Query<(), Or<(With<UninitializedMenuItem>, With<MenuContentsProcessing>)>>,
    // Query menu items which do not already have a layout component
    item_query: Query<(Entity, &MenuItem), Without<Node>>,
    mut commands: Commands,
) {
    // Wait until all components are initialized
    if !processing_query.is_empty() {
        return;
    }

    for (item_entity, item) in item_query {
        commands.entity(item_entity).insert(Node {
            flex_direction: FlexDirection::ColumnReverse,

            width: percent(100),
            padding: UiRect::px(
                CONTENT_INNER_PADDING,
                CONTENT_PADDING,
                CONTENT_PADDING,
                CONTENT_PADDING,
            ),

            ..Default::default()
        });

        let palette = BackgroundInteractionPalette {
            none: TRANSPARENT_COLOR,
            hovered: HOVERED_COLOR,
            pressed: PRESSED_COLOR,
        };

        let button = {
            bsn! [
                {
                    match item {
                        MenuItem::Folder(_) => Box::new(bsn! [{
                            button::with_text_inline(
                                item.name(),
                                ButtonWithTextOptions {
                                    font_size: SMALL_FONT_SIZE,
                                    color: HEADER_TEXT,
                                    width: percent(100),
                                    height: Val::Auto,
                                    ..default()
                                },
                                palette,
                                folder_button_clicked,
                            )
                        }]) as Box<dyn Scene>,
                        MenuItem::File(_, _, _) => Box::new(bsn! [{
                            button::with_text_inline(
                                item.name(),
                                ButtonWithTextOptions {
                                    font_size: SMALL_FONT_SIZE,
                                    color: HEADER_TEXT,
                                    width: percent(100),
                                    height: Val::Auto,
                                    ..default()
                                },
                                palette,
                                file_button_clicked,
                            )
                        }]) as Box<dyn Scene>,
                    }
                }
                Node {
                    width: percent(100),
                    margin: UiRect::right(px(BROWSER_BUTTON_MARGIN)),
                    padding: UiRect::horizontal(px(BROWSER_BUTTON_PADDING)),
                    justify_content: JustifyContent::FlexStart,
                }
            ]
        };

        let button = commands.spawn_scene(button).id();
        commands.entity(item_entity).add_child(button);
    }
}

fn folder_button_clicked(_: On<Pointer<Click>>) {}

fn file_button_clicked(
    event: On<Pointer<Click>>,
    parent_query: Query<&ChildOf>,
    file_query: Query<&MenuItem>,
    mut file_manager: ResMut<FileManager>,
    channel_set: FileTaskChannelSet,
) {
    let Ok(menu_item) = parent_query.get(event.entity).map(ChildOf::get) else {
        error!("Failed to get menu item entity");
        return;
    };

    let Ok(menu_item) = file_query.get(menu_item) else {
        error!("Failed to get menu item component from entity");
        return;
    };

    let MenuItem::File(_, loc, file_kind) = menu_item else {
        error!("Menu item was not a file!");
        return;
    };

    match event.button {
        PointerButton::Primary => file_manager.open(loc.clone(), *file_kind, true, &channel_set),
        PointerButton::Secondary => {}
        PointerButton::Middle => file_manager.open(loc.clone(), *file_kind, false, &channel_set),
    }
}
