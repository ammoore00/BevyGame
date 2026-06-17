use crate::datagen_api::animation::{AnimationCodec, AnimationResource};
use crate::datagen_api::assets::{CharacterCodec, CharacterResource};
use crate::datagen_api::attack::{AttackCodec, AttackResource};
use crate::dev_tools::editor::file_manager::{EditorFile, EditorResourceKind, FileKind, FileManager};
use crate::menus::font::FontBuilder;
use crate::screens::Screen;
use crate::theme::widget::{UiAssets, UiResources};
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;
use serde::de::DeserializeOwned;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_properties_view,
        )
            .run_if(in_state(Screen::Editor))
    );
}

#[derive(Component, Debug, Clone, Default, Copy)]
struct PropertiesScreen;

pub(super) fn spawn_details_screen(
    mut commands: Commands,
) -> Entity {
    commands.spawn((
        PropertiesScreen,
        Node {
            width: percent(100),
            height: percent(100),

            ..Default::default()
        }
    )).id()
}

fn update_properties_view(
    // Query for the current properties screen
    properties_query: Query<(
        Entity,
        &PropertiesInner
    )>,
    file_manager: Res<FileManager>,
    mut ui_resources: UiResources,
    mut commands: Commands,
) {
    // Get the current properties view
    let properties = match properties_query.single() {
        Ok((entity, inner)) => Some((entity, inner)),
        Err(QuerySingleError::NoEntities(_)) => {
            None
        },
        Err(QuerySingleError::MultipleEntities(err)) => {
            error!("Found multiple properties entities: {err}\nDespawning all entities.");
            for (entity, _) in properties_query.iter() {
                commands.entity(entity).despawn();
            }
            return;
        },
    };

    // If there is no active file, despawn the properties entity if it exists
    let Some(active_file) = file_manager.active_file() else {
        if let Some((properties_entity, _)) = properties {
            commands.entity(properties_entity).despawn();
        }
        return;
    };

    let should_spawn = if let Some((properties_entity, PropertiesInner(active_file_in_properties))) = properties {
        // If there is an active file, and it does not match the current properties screen,
        // despawn it and flag that we should spawn a new properties editor
        if active_file_in_properties != active_file {
            commands.entity(properties_entity).despawn();
            true
        }
        // If the properties screen matches the active file, do not spawn a new properties editor
        else { false }
    }
    // If there is an active file, but no properties screen, flag that we should spawn a new properties editor
    else { true };

    if should_spawn {
        active_file.spawn_properties_editor(
            &mut ui_resources,
            commands,
        );
    }
}

#[derive(Component, Debug, Clone)]
struct PropertiesInner(EditorFile);

impl EditorFile {
    fn spawn_properties_editor(
        &self,
        ui_resources: &mut UiResources,
        mut commands: Commands,
    ) -> Entity {
        match self.kind() {
            FileKind::Character => commands.spawn((
                self.properties_bundle::<CharacterCodec>(ui_resources),
                PropertiesInner(self.clone()),
            )).id(),
            FileKind::Animation => commands.spawn((
                self.properties_bundle::<AnimationCodec>(ui_resources),
                PropertiesInner(self.clone()),
            )).id(),
            FileKind::Attack => commands.spawn((
                self.properties_bundle::<AttackCodec>(ui_resources),
                PropertiesInner(self.clone()),
            )).id(),
        }
    }

    fn properties_bundle<Codec>(
        &self,
        ui_assets: &mut UiResources,
    ) -> impl Bundle
    where
        Codec: EditorCodec,
    {
        let loc = self.loc_typed::<Codec::Resource>();
        
        // Load asset from disk or create default
    }
}

/// Trait for codecs which can be represented in the editor.
pub trait EditorCodec: DeserializeOwned + Default + Clone + Send + Sync + 'static {
    type Resource: EditorResourceKind;
    
    fn properties_bundle(
        // Method must consume self to satisfy lifetimes for opaque return type
        self,
        ui_assets: &UiAssets,
        font_builder: &FontBuilder,
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> impl Bundle;
}

impl EditorCodec for CharacterCodec {
    type Resource = CharacterResource;
    
    fn properties_bundle(
        self,
        ui_assets: &UiAssets,
        font_builder: &FontBuilder,
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> impl Bundle {}
}

impl EditorCodec for AnimationCodec {
    type Resource = AnimationResource;
    
    fn properties_bundle(
        self,
        ui_assets: &UiAssets,
        font_builder: &FontBuilder,
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> impl Bundle {}
}

impl EditorCodec for AttackCodec {
    type Resource = AttackResource;
    
    fn properties_bundle(
        self,
        ui_assets: &UiAssets,
        font_builder: &FontBuilder,
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> impl Bundle {}
}