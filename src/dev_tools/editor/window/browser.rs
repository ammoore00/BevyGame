use crate::data::registry::{ResolvedSystemRegistry, SystemRegistry};
use crate::data::{ResolvableResource, ResourceLocation, ResourceType};
use crate::datagen_api::animation::AnimationResource;
use crate::datagen_api::assets::CharacterResource;
use crate::datagen_api::attack::AttackResource;
use crate::menus::font::FontBuilder;
use crate::screens::Screen;
use crate::theme::palette::{BUTTON_TEXT, HEADER_TEXT};
use crate::theme::widget;
use crate::theme::widget::{styled_button, ButtonStyle, UiAssets, MEDIUM_FONT_SIZE, SMALL_FONT_SIZE};
use bevy::ecs::query::QuerySingleError;
use bevy::ecs::relationship::Relationship;
use bevy::ecs::system::{IntoObserverSystem, SystemParam};
use bevy::prelude::*;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;

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
                    update_menu_contents_from_resolved_registry::<AnimationMenu>,
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
            handle_menu_browser_button_interaction,
        )
            .chain()
            .run_if(in_state(Screen::Editor))
    );
}

#[derive(Component, Debug, Clone, Default, Copy)]
struct FileBrowser;

pub(super) fn spawn_file_browser(
    ui_assets: &UiAssets,
    font_builder: &FontBuilder,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    mut commands: Commands,
) -> Entity {
    let browser = commands.spawn((
        FileBrowser,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2.),

            width: percent(100),
            height: percent(100),

            ..Default::default()
        },
    )).id();

    let characters = collapsible_menu(
        ui_assets,
        texture_atlas_layouts,
        "Characters",
        font_builder,
        24.,
        commands.reborrow(),
    );
    commands.entity(characters).insert(CharacterMenu);
    commands.entity(browser).add_child(characters);

    let animations = collapsible_menu(
        ui_assets,
        texture_atlas_layouts,
        "Animations",
        font_builder,
        24.,
        commands.reborrow(),
    );
    commands.entity(animations).insert(AnimationMenu);
    commands.entity(browser).add_child(animations);

    let attacks = collapsible_menu(
        ui_assets,
        texture_atlas_layouts,
        "Attacks",
        font_builder,
        24.,
        commands.reborrow(),
    );
    commands.entity(attacks).insert(AttackMenu);
    commands.entity(browser).add_child(attacks);

    browser
}

#[derive(Component, Debug, Clone, Copy)]
struct Collapsed(bool);
impl Default for Collapsed { fn default() -> Self { Self(true) } }

fn collapsible_menu(
    ui_assets: &UiAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    text: impl Into<String>,
    font_builder: &FontBuilder,
    font_size: f32,
    mut commands: Commands,
) -> Entity {
    let menu = commands.spawn((
        Collapsed::default(),
        Node {
            flex_direction: FlexDirection::Column,

            width: percent(100),

            ..Default::default()
        },
    )).id();

    let menu_inner = commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(8),

            width: percent(100),

            ..Default::default()
        },
    )).id();
    commands.entity(menu).add_child(menu_inner);

    let button = commands.spawn((
        styled_button(
            ui_assets,
            texture_atlas_layouts,
            2,
            |
                event: On<Pointer<Click>>,
                parent_query: Query<&ChildOf>,
                mut commands: Commands,
                mut menu_query: Query<&mut Collapsed>,
            | {
                let Ok(button_root) = parent_query.get(event.entity).map(ChildOf::get) else {
                    error!("Failed to get button root");
                    return;
                };

                let Ok(menu_inner) = parent_query.get(button_root).map(ChildOf::get) else {
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
            },
            ButtonStyle::ArrowRight,
        ),
    )).id();
    commands.entity(menu_inner).add_child(button);

    let text = commands.spawn(widget::text(
        text,
        font_builder,
        font_size,
        HEADER_TEXT,
    )).id();
    commands.entity(menu_inner).add_child(text);

    menu
}

trait MenuContentsKind: Component + Debug {
    type ResourceKind: ResourceType;
}
trait ResolvedMenuContentsKind: MenuContentsKind {
    type ResolvableResourceKind: ResolvableResource;
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
impl ResolvedMenuContentsKind for AnimationMenu {
    type ResolvableResourceKind = AnimationResource;
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
    _phantom_data: PhantomData<T>
}
impl<T: MenuContentsKind> MenuContents<T> {
    fn new() -> Self {
        Self { _phantom_data: PhantomData }
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
    menu_query: Query<
        (
            Entity,
            &Collapsed
        ),
        With<ContentKind>
    >,
    contents_query: Query<
        Entity,
        With<MenuContents<ContentKind>>
    >,
    font_builder: FontBuilder,
    mut commands: Commands,
)
where
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
                let contents = commands.spawn((
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
                )).id();
                commands.entity(menu).add_child(contents);
            }
        }
        Err(QuerySingleError::MultipleEntities(_)) => error!("Multiple entities found for menu contents"),
    }
}

trait MenuRegistryAccessor<T: ResourceType>: SystemParam {
    type AssetKind: Asset;
    fn iter(&self) -> impl Iterator<Item = (ResourceLocation<T>, Handle<Self::AssetKind>)>;
}

impl<'w, T: ResourceType> MenuRegistryAccessor<T> for SystemRegistry<'w, T> {
    type AssetKind = T::AssetType;
    fn iter(&self) -> impl Iterator<Item = (ResourceLocation<T>, Handle<Self::AssetKind>)> {
        self.registry().iter()
            .map(|(location, handle)| (location.clone(), handle.clone()))
    }
}
impl<'w, T: ResolvableResource> MenuRegistryAccessor<T> for ResolvedSystemRegistry<'w, T> {
    type AssetKind = T::ResolvedAssetType;
    fn iter(&self) -> impl Iterator<Item = (ResourceLocation<T>, Handle<Self::AssetKind>)> {
        self.resolved_registry().iter()
            .map(|(location, handle)| (location.clone(), handle.clone()))
    }
}

fn update_menu_contents_from_registry<
    ContentKind: MenuContentsKind,
> (
    contents_query: Query<
        (
            Entity,
            Option<&MenuContentsUninitialized>,
        ),
        With<MenuContents<ContentKind>>
    >,
    registry: SystemRegistry<ContentKind::ResourceKind>,
    commands: Commands,
) {
    update_menu_contents_inner::<ContentKind, _, _>(contents_query, registry, commands)
}

fn update_menu_contents_from_resolved_registry<
    ContentKind: ResolvedMenuContentsKind,
> (
    contents_query: Query<
        (
            Entity,
            Option<&MenuContentsUninitialized>,
        ),
        With<MenuContents<ContentKind>>
    >,
    registry: ResolvedSystemRegistry<ContentKind::ResolvableResourceKind>,
    commands: Commands,
) {
    update_menu_contents_inner::<ContentKind, _, _>(contents_query, registry, commands)
}

/// Find all resource locations to populate the given menu
/// and adds them to the list in a hierarchical folder structure
fn update_menu_contents_inner<ContentKind, Registry, Resource>(
    contents_query: Query<
        (
            Entity,
            Option<&MenuContentsUninitialized>,
        ),
        With<MenuContents<ContentKind>>
    >,
    registry: Registry,
    mut commands: Commands,
)
where
    ContentKind: MenuContentsKind,
    Registry: MenuRegistryAccessor<Resource>,
    Resource: ResourceType,
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

    if uninitialized.is_some() {
        for (loc, _) in registry.iter() {
            let item = commands.spawn(UninitializedMenuItem(loc.as_local_path().to_path_buf())).id();
            commands.entity(contents).add_child(item);
        }
    }
}

#[derive(Component, Debug, Clone)]
struct UninitializedMenuItem(PathBuf);

#[derive(Component, Debug, Clone)]
enum MenuItem {
    Folder(String),
    File(String),
}
impl MenuItem {
    fn name(&self) -> &str {
        match self {
            MenuItem::Folder(name) => name,
            MenuItem::File(name) => name,
        }
    }
}

/// Process menu items marked as uninitialized and populate their children
fn update_menu_items(
    // Query for the item itself which we want to update
    items_query: Query<(
        Entity,
        &UninitializedMenuItem,
        &ChildOf,
    )>,
    // Query for getting the parent of the item
    parent_query: Query<
        (
            Entity,
            &Children,
            Option<&MenuContentsUninitialized>,
        ),
        Or<(With<MenuItem>, With<MenuContentsUntypedMarker>)>
    >,
    // Query for getting the siblings of the item
    sibling_query: Query<(
        Entity,
        &MenuItem,
    )>,
    mut commands: Commands,
) {
    // Only process one item per frame in order to prevent duplicate items
    // TODO: Profile this to make sure it isn't taking too long
    for (item_entity, item, parent) in items_query.iter().take(1) {
        let path = item.0.clone();
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
            commands.entity(parent_entity).remove::<MenuContentsUninitialized>();
            commands.entity(parent_entity).insert(MenuContentsProcessing);
        }

        let mut siblings = sibling_query.iter()
            .filter(|(sibling_entity, _)| siblings.contains(sibling_entity));

        // Check if the current component is the last one, and thus we are at the final file
        let is_last = components.peek().is_none();

        // See if an item already exists for this component
        let existing = siblings.find(|(_, sibling_item)| {
            match (sibling_item, is_last) {
                (MenuItem::Folder(name), false)
                | (MenuItem::File(name), true) => component == name,
                _ => false
            }
        });

        match (is_last, existing) {
            //If we are on the actual file:
            // If it already exists, despawn the uninitialized item
            (true, Some(_)) => {
                commands.entity(item_entity).despawn();
            },
            // Otherwise, replace the uninitialized reference with an item
            (true, None) => {
                commands.entity(item_entity).remove::<UninitializedMenuItem>();
                commands.entity(item_entity).insert(MenuItem::File(component.to_string()));
            },
            //If we are still in a directory:
            // If the child already exists, despawn the uninitialized entity,
            // then add a new uninitialized one under the existing match
            (false, Some((existing, _))) => {
                commands.entity(item_entity).despawn();
                let child = commands.spawn(UninitializedMenuItem(remaining_path)).id();
                commands.entity(existing).add_child(child);
            },
            // Otherwise, replace the uninitialized reference with a folder,
            // then add an uninitialized child entity
            (false, None) => {
                commands.entity(item_entity).remove::<UninitializedMenuItem>();
                commands.entity(item_entity).insert(MenuItem::Folder(component.to_string()));
                let child = commands.spawn(UninitializedMenuItem(remaining_path)).id();
                commands.entity(item_entity).add_child(child);
            },
        }
    }
}

/// Once all processing is done, finalize the menu items
fn finalize_menu_items(
    // Find if there are any items marked as uninitialized
    uninitialized_query: Query<(), With<UninitializedMenuItem>>,
    // Query children of menu tree roots
    menu_query: Query<
        Entity,
        With<MenuContentsProcessing>
    >,
    mut commands: Commands
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

fn render_menu_items(
    // Only process rendering information if everything is done processing
    processing_query: Query<(), Or<(
        With<UninitializedMenuItem>,
        With<MenuContentsProcessing>
    )>>,
    // Query menu items which do not already have a layout component
    item_query: Query<
        (
            Entity,
            &MenuItem,
        ),
        Without<Node>
    >,
    font_builder: FontBuilder,
    mut commands: Commands
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

        let button = match item {
            MenuItem::Folder(_) => commands.spawn(browser_button(
                item.name(),
                &font_builder,
                HEADER_TEXT,
                folder_button_clicked,
            )).id(),
            MenuItem::File(_) => commands.spawn(browser_button(
                item.name(),
                &font_builder,
                BUTTON_TEXT,
                file_button_clicked,
            )).id(),
        };
        commands.entity(item_entity).add_child(button);
    }
}

#[derive(Component, Debug, Clone)]
struct BrowserButton;
#[derive(Component, Debug, Clone)]
struct BrowserButtonInner;

const TRANSPARENT_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
const HOVERED_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.2);
const PRESSED_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);

const BROWSER_BUTTON_PADDING: f32 = 6.;
const BROWSER_BUTTON_MARGIN: f32 = 12.;

fn browser_button<E, B, M, I>(
    text: impl AsRef<str>,
    font_builder: &FontBuilder,
    color: Color,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.as_ref();
    let action = IntoObserverSystem::into_system(action);

    let text_bundle = (
        Text(text.to_string()),
        font_builder.with_size(SMALL_FONT_SIZE),
        TextLayout {
            justify: Justify::Left,
            ..Default::default()
        },
        TextColor(color),
    );

    (
        BrowserButton,
        Node {
            ..Default::default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((
                BrowserButtonInner,
                Button,
                BackgroundColor(TRANSPARENT_COLOR),
                Node {
                    width: percent(100),
                    margin: UiRect::right(px(BROWSER_BUTTON_MARGIN)),
                    padding: UiRect::horizontal(px(BROWSER_BUTTON_PADDING)),

                    ..Default::default()
                },
                children![text_bundle],
            )).observe(action);
        })),
    )
}
fn handle_menu_browser_button_interaction(
    button_query: Query<
        (&Interaction, &mut BackgroundColor, &BrowserButtonInner),
        (Changed<Interaction>, With<BrowserButtonInner>),
    >,
) {
    for (interaction, mut background_color, _) in button_query {
        *background_color = match interaction {
            Interaction::Pressed => BackgroundColor(PRESSED_COLOR),
            Interaction::Hovered => BackgroundColor(HOVERED_COLOR),
            Interaction::None => BackgroundColor(TRANSPARENT_COLOR),
        }
    }
}

fn folder_button_clicked(
    _: On<Pointer<Click>>,
) {}

fn file_button_clicked(
    _: On<Pointer<Click>>,
) {}