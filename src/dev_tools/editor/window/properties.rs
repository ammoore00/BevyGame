use std::fs::File;
use crate::datagen_api::animation::{AnimationCodec, AnimationResource};
use crate::datagen_api::assets::{CharacterCodec, CharacterResource};
use crate::datagen_api::attack::{AttackCodec, AttackResource};
use crate::dev_tools::editor::file_manager::{EditorFile, EditorFileComponent, EditorFileContent, EditorResourceKind, FileKind, FileManager};
use crate::screens::Screen;
use crate::theme::widget::{UiResources, SMALL_FONT_SIZE};
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;
use bevy_ui_text_input::{TextInputMode, TextInputNode};
use serde::de::DeserializeOwned;
use crate::theme::palette::HEADER_TEXT;
use crate::theme::widget;

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
    parent_query: Query<
        Entity,
        With<PropertiesScreen>
    >,
    // Query for the current properties screen
    properties_query: Query<(
        Entity,
        &PropertiesInner
    )>,
    file_query: Query<(
        &EditorFileComponent,
        &EditorFileContent,
    )>,
    file_manager: Res<FileManager>,
    mut ui_resources: UiResources,
    mut commands: Commands,
) {
    // Get the current properties view
    let properties = match properties_query.single() {
        Ok((entity, inner)) => {
            Some((entity, inner))
        },
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
            info!("Despawning properties editor for file: {}", active_file_in_properties.loc());
            info!("Spawning new properties editor for file: {}", active_file.loc());
            true
        }
        // If the properties screen matches the active file, do not spawn a new properties editor
        else {
            false
        }
    }
    // If there is an active file, but no properties screen, flag that we should spawn a new properties editor
    else {
        info!("No editor exists, spawning new properties editor for file: {}", active_file.loc());
        true
    };

    if should_spawn {
        info!("Attempting to spawn editor for file: {}", active_file.loc());

        let content = file_query.into_iter().find(|(file, _)| {
            &file.0 == active_file
        });

        if let Some((_, content)) = content {
            let Ok(parent) = parent_query.single() else {
                error!("Could not obtain primary properties screen entity");
                return;
            };

            info!("Spawning new properties editor for file: {}", active_file.loc());
            let properties = active_file.spawn_properties_editor(
                content,
                &mut ui_resources,
                commands.reborrow(),
            );

            commands.entity(parent).add_child(properties);
        }
    }
}

#[derive(Component, Debug, Clone)]
struct PropertiesInner(EditorFile);

impl EditorFile {
    fn spawn_properties_editor(
        &self,
        content: &EditorFileContent,
        ui_resources: &mut UiResources,
        mut commands: Commands,
    ) -> Entity {
        let shared_bundle = (
            Node::default(),
            PropertiesInner(self.clone()),
        );

        match self.kind() {
            FileKind::Character => commands.spawn((
                self.properties_bundle::<CharacterCodec>(content.get_character_codec(), ui_resources),
                shared_bundle,
            )).id(),
            FileKind::Animation => commands.spawn((
                self.properties_bundle::<AnimationCodec>(content.get_animation_codec(), ui_resources),
                shared_bundle,
            )).id(),
            FileKind::Attack => commands.spawn((
                self.properties_bundle::<AttackCodec>(content.get_attack_codec(), ui_resources),
                shared_bundle,
            )).id(),
        }
    }

    fn properties_bundle<Codec: EditorCodec>(
        &self,
        content: Codec,
        ui_resources: &mut UiResources,
    ) -> impl Bundle {
        content.properties_bundle(ui_resources)
    }
}

/// Trait for codecs which can be represented in the editor.
pub trait EditorCodec: DeserializeOwned + Default + Clone + Send + Sync + 'static {
    type Resource: EditorResourceKind;

    const FILE_CONTENT_FN: fn(Self) -> EditorFileContent;
    const FILE_TYPE: FileKind;

    // Method must consume self to satisfy lifetimes for opaque return type
    fn properties_bundle(self, ui_resources: &mut UiResources) -> impl Bundle;
}

impl EditorCodec for CharacterCodec {
    type Resource = CharacterResource;
    const FILE_CONTENT_FN: fn(Self) -> EditorFileContent = EditorFileContent::Character;
    const FILE_TYPE: FileKind = FileKind::Character;

    fn properties_bundle(self, ui_resources: &mut UiResources) -> impl Bundle {}
}

impl EditorCodec for AnimationCodec {
    type Resource = AnimationResource;
    const FILE_CONTENT_FN: fn(Self) -> EditorFileContent = EditorFileContent::Animation;
    const FILE_TYPE: FileKind = FileKind::Animation;

    fn properties_bundle(self, ui_resources: &mut UiResources) -> impl Bundle {
        children![
            text_input(ui_resources, UVec2::new(300, 50), "Animation")
        ]
    }
}

impl EditorCodec for AttackCodec {
    type Resource = AttackResource;
    const FILE_CONTENT_FN: fn(Self) -> EditorFileContent = EditorFileContent::Attack;
    const FILE_TYPE: FileKind = FileKind::Attack;
    
    fn properties_bundle(self, ui_resources: &mut UiResources) -> impl Bundle {}
}

fn text_input(
    ui_resources: &mut UiResources,
    size: UVec2,
    label: &str,
) -> impl Bundle {
    (
        Node {
            ..Default::default()
        },
        children![
            widget::text(label, &ui_resources.font_builder, SMALL_FONT_SIZE, HEADER_TEXT),
            (
                TextInputNode {
                    mode: TextInputMode::SingleLine,
                    ..Default::default()
                },
                Node {
                    width: px(size.x),
                    height: px(size.y),
                    ..default()
                },
            )
        ],
    )
}