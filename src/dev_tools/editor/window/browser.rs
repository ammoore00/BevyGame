use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;
use bevy::ecs::query::QuerySingleError;
use bevy::ecs::relationship::Relationship;
use bevy::ecs::system::SystemParam;
use crate::theme::palette::HEADER_TEXT;
use crate::theme::widget::{styled_button, ButtonStyle, UiAssets};
use bevy::prelude::*;
use crate::data::registry::{ResolvedSystemRegistry, SystemRegistry};
use crate::data::{ResolvableResource, ResourceLocation, ResourceType};
use crate::datagen_api::animation::AnimationResource;
use crate::datagen_api::assets::CharacterResource;
use crate::datagen_api::attack::AttackResource;
use crate::menus::font::FontBuilder;
use crate::screens::Screen;
use crate::theme::widget;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
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
        )
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
struct MenuContents<T: MenuContentsKind> {
    contents: MenuItem<T>,
}
impl<T: MenuContentsKind> MenuContents<T> {
    fn new() -> Self {
        Self {
            contents: MenuItem::folder("#root".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
enum MenuItem<T: MenuContentsKind> {
    Folder {
        name: String,
        children: Vec<MenuItem<T>>,
        _phantom_data: PhantomData<T>
    },
    Item(String),
}
impl<T: MenuContentsKind> MenuItem<T> {
    fn folder(name: String) -> Self {
        Self::Folder {
            name,
            children: Vec::new(),
            _phantom_data: PhantomData,
        }
    }

    fn create_all(&mut self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        
        match self {
            MenuItem::Folder{ children, .. } => {
                let mut components = path.components().peekable();
                let Some(component) = components.next() else {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "cannot create empty path"))
                };

                // Check if this is the last item
                let is_last = components.peek().is_none();

                let component = component.as_os_str().to_string_lossy();
                let component = component.as_ref();

                // Find if there is already a matching entry
                let child = children.iter_mut()
                    .find(|child| {
                        match child {
                            MenuItem::Folder{ name, .. } => *name == component,
                            MenuItem::Item(name) => is_last && *name == component,
                        }
                    });

                let remaining_path = path.strip_prefix(component).unwrap();
                match (is_last, child) {
                    //If we are on the actual file:
                    // If it already exists, do nothing
                    (true, Some(_)) => {},
                    // Otherwise, create it and finish
                    (true, None) => {
                        let child = MenuItem::Item(component.to_string());
                        children.push(child);
                    },
                    // If we are still in a directory:
                    // If the child already exists, recursively create the remaining path
                    (false, Some(child)) => child.create_all(remaining_path)?,
                    // Otherwise, create the child, then recursively create the remaining path
                    (false, None) => {
                        let mut child = MenuItem::folder(component.to_string());
                        child.create_all(remaining_path)?;
                        children.push(child);
                    },
                }

                Ok(())
            },
            MenuItem::Item(_) => {
                Err(std::io::Error::new(std::io::ErrorKind::NotADirectory, "Cannot create sub-items on items"))
            }
        }
    }
}

const CONTENT_START_PADDING: f32 = 40.;
const CONTENT_PADDING: f32 = 4.;

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
                    MenuContents::<ContentKind>::new(),
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
                    children![
                        Text("Content".to_string()),
                        font_builder.with_size(24.),
                        Node {
                            ..Default::default()
                        },
                    ]
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
    contents_query: Query<&mut MenuContents<ContentKind>>,
    registry: SystemRegistry<ContentKind::ResourceKind>,
) {
    update_menu_contents_inner::<ContentKind, _, _>(contents_query, registry)
}

fn update_menu_contents_from_resolved_registry<
    ContentKind: ResolvedMenuContentsKind,
> (
    contents_query: Query<&mut MenuContents<ContentKind>>,
    registry: ResolvedSystemRegistry<ContentKind::ResolvableResourceKind>,
) {
    update_menu_contents_inner::<ContentKind, _, _>(contents_query, registry)
}

/// Find all resource locations to populate the given menu
/// and adds them to the list in a hierarchical folder structure
fn update_menu_contents_inner<ContentKind, Registry, Resource>(
    mut contents_query: Query<&mut MenuContents<ContentKind>>,
    registry: Registry,
)
where
    ContentKind: MenuContentsKind,
    Registry: MenuRegistryAccessor<Resource>,
    Resource: ResourceType,
{
    // TODO: Figure out a way to make this work with change detection?

    let single = contents_query.single_mut();
    let mut contents = match single {
        Ok(single) => single,
        Err(QuerySingleError::NoEntities(_)) => {
            return;
        }
        Err(QuerySingleError::MultipleEntities(err)) => {
            error!("Failed to get menu contents entity: {}", err);
            return;
        }
    };

    for (loc, _resource) in registry.iter() {
        let result = contents.contents.create_all(loc.as_local_path());
        if let Err(err) = result {
            error!("Failed to create menu contents: {}", err);
        }
    }
}