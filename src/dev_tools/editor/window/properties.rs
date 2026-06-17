use crate::datagen_api::animation::{AnimationCodec, AnimationResource};
use crate::datagen_api::assets::{CharacterCodec, CharacterResource};
use crate::datagen_api::attack::{AttackCodec, AttackResource};
use crate::dev_tools::editor::file_manager::{EditorFile, EditorResourceKind, FileKind, FileManager};
use crate::menus::font::FontBuilder;
use crate::screens::Screen;
use crate::theme::widget::UiAssets;
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
    properties_query: Query<
        Entity,
        With<PropertiesInner>
    >,
    file_manager: Res<FileManager>,
    ui_assets: Res<UiAssets>,
    font_builder: FontBuilder,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    // Get the current properties view
    let properties_entity = match properties_query.single() {
        Ok(entity) => Some(entity),
        Err(QuerySingleError::NoEntities(_)) => {
            None
        },
        Err(QuerySingleError::MultipleEntities(err)) => {
            error!("Found multiple properties entities: {err}\nDespawning all entities.");
            for entity in properties_query.iter() {
                commands.entity(entity).despawn();
            }
            return;
        },
    };

    // If there is no active file, despawn the properties entity if it exists
    let Some(active_file) = file_manager.active_file() else {
        if let Some(properties_entity) = properties_entity {
            commands.entity(properties_entity).despawn();
        }
        return;
    };

    active_file.read().unwrap().spawn_properties_editor(
        &ui_assets,
        &font_builder,
        &mut texture_atlas_layouts,
        commands,
    );
}

#[derive(Component, Debug, Clone, Default, Copy)]
struct PropertiesInner;

impl EditorFile {
    fn spawn_properties_editor(
        &self,
        ui_assets: &UiAssets,
        font_builder: &FontBuilder,
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
        mut commands: Commands,
    ) -> Entity {
        match self.kind() {
            FileKind::Character => commands.spawn((
                self.properties_bundle::<CharacterCodec>(ui_assets, font_builder, texture_atlas_layouts),
                PropertiesInner,
            )).id(),
            FileKind::Animation => commands.spawn((
                self.properties_bundle::<AnimationCodec>(ui_assets, font_builder, texture_atlas_layouts),
                PropertiesInner,
            )).id(),
            FileKind::Attack => commands.spawn((
                self.properties_bundle::<AttackCodec>(ui_assets, font_builder, texture_atlas_layouts),
                PropertiesInner,
            )).id(),
        }
    }

    fn properties_bundle<Codec>(
        &self,
        ui_assets: &UiAssets,
        font_builder: &FontBuilder,
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
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