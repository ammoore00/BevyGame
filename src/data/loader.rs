use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use bevy::reflect::erased_serde::__private::serde::Deserializer;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use crate::data::registry::ResourceRegistry;
use crate::data::{ResourceLocation, ResourceType};
use crate::StartupSystems;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, load_assets.in_set(StartupSystems::LoadAssets));
}

fn load_assets(world: &mut World) {
    let loader = world.resource::<GameAssetLoader>();
    let jobs = loader.jobs.clone();
    jobs.iter().for_each(|job| job.load(world).expect("Failed to load assets"));
}

/// Resource which holds a list of jobs to load assets
/// The jobs retrieve the `ResourceRegistry` from the World,
/// load each asset, then insert them into the registry
#[derive(Default, Resource)]
pub struct GameAssetLoader {
    jobs: Vec<Arc<dyn RegistryLoader>>,
}
impl GameAssetLoader {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn add_job<T: ResourceType>(&mut self) {
        self.jobs.push(Arc::new(LoaderJob::<T>::default()));
    }
}

pub trait LoaderJobManager {
    /// Adds a job to the asset loader which will load all assets in the registry
    fn add_resource_registry<T: ResourceType>(&mut self);
    /// Adds a job to the asset loader with a pre-filled manifest
    fn add_registry_with_manifest<T: ResourceType>(&mut self, manifest: Vec<ResourceLocation<T>>);
    /// Adds a job to the asset loader which will discover all assets in the registry automatically
    fn add_registry_with_discovery<T: ResourceType>(&mut self);
}

impl LoaderJobManager for App {
    fn add_resource_registry<T: ResourceType>(&mut self) {
        let world = self.world_mut();
        world.insert_resource(ResourceRegistry::<T>::default());

        if !world.contains_resource::<GameAssetLoader>() {
            world.insert_resource(GameAssetLoader::new());
        }

        let mut asset_loader = world.resource_mut::<GameAssetLoader>();
        asset_loader.add_job::<T>();
    }

    fn add_registry_with_manifest<T: ResourceType>(&mut self, manifest: Vec<ResourceLocation<T>>) {
        self.add_resource_registry::<T>();
        let world = self.world_mut();
        let mut registry = world.resource_mut::<ResourceRegistry<T>>();
        registry.extend_manifest(manifest);
    }

    // TODO: Eventually I want to be able to load assets from multiple places (e.g. mod files)
    //       This will require a way to check all places, not just the normal assets folder
    fn add_registry_with_discovery<T: ResourceType>(&mut self) {
        // Find all namespaces currently available
        let Ok(namespaces) = std::fs::read_dir("./assets/") else {
            return;
        };

        let mut manifest = Vec::new();

        // For each namespace, find the root dir of the resource type
        // and load any files found within
        for namespace in namespaces.flatten() {
            // Make sure the namespace is actually a directory
            let namespace_path = &namespace.path();
            if !namespace_path.is_dir() {
                continue;
            }

            let namespace = namespace_path.strip_prefix("./assets").unwrap();

            // Find all files under the namespace
            WalkDir::new(namespace_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().strip_prefix("./assets").unwrap().to_path_buf())
                .filter(|path| path.strip_prefix(namespace).unwrap().starts_with(T::ROOT_DIR))
                .filter_map(|path| ResourceLocation::from_path(path).ok())
                .for_each(|location| manifest.push(location));
        }

        self.add_registry_with_manifest::<T>(manifest);
    }
}

trait RegistryLoader: Send + Sync + 'static {
    fn load(&self, world: &mut World) -> Result<(), LoaderError>;
}

#[derive(Debug)]
struct LoaderJob<T: ResourceType> {
    phantom_data: PhantomData<T>,
}
impl<T: ResourceType> Default for LoaderJob<T> {
    fn default() -> Self {
        Self {
            phantom_data: Default::default()
        }
    }
}
impl<T: ResourceType> RegistryLoader for LoaderJob<T> {
    /// Iterate through all registered assets for the associated registry and loads them
    fn load(&self, world: &mut World) -> Result<(), LoaderError> {
        let asset_server = world.resource::<AssetServer>();

        let mut assets = HashMap::new();

        let registry = world.resource::<ResourceRegistry<T>>();
        let manifest = registry.manifest();
        manifest.iter().for_each(|loc| {
            let path = loc.as_path();
            let asset = asset_server.load::<T::AssetType>(path);
            assets.insert(loc.clone(), asset);
        });

        let mut registry = world.resource_mut::<ResourceRegistry<T>>();
        assets.into_iter().for_each(|(loc, asset)| registry.register_asset(loc, asset));

        Ok(())
    }
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
}

#[derive(TypePath)]
pub struct RonAssetLoader<Codec, AssetType>
where
    Codec: DeserializeOwned + TypePath + Into<AssetType> + Send + Sync + 'static,
    AssetType: Asset + Send + Sync + 'static,
{
    phantom_data: PhantomData<(Codec, AssetType)>,
}
impl<Codec, AssetType> Default for RonAssetLoader<Codec, AssetType>
where
    Codec: DeserializeOwned + TypePath + Into<AssetType> + Send + Sync + 'static,
    AssetType: Asset + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}
impl<Codec, AssetType> AssetLoader for RonAssetLoader<Codec, AssetType>
where
    Codec: DeserializeOwned + TypePath + Into<AssetType> + Send + Sync + 'static,
    AssetType: Asset + Send + Sync + 'static,
{
    type Asset = AssetType;
    type Settings = ();
    type Error = RonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let codec = ron::de::from_bytes::<Codec>(&bytes)?;
        Ok(codec.into())
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RonLoaderError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RON parse error: {0}")]
    Ron(#[from] ron::error::SpannedError),
}

/// Wrapper around Option<T> to be used as an optional value within a codec
/// Used instead of Option to avoid Option enum values being included in
/// output files
#[derive(Default)]
pub struct Maybe<T: Serialize>(pub Option<T>);
impl<T: Serialize> Maybe<T> {
    pub fn into_inner(self) -> Option<T> {
        self.0
    }
}

impl<T: Serialize> Deref for Maybe<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Serialize> Serialize for Maybe<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            Some(value) => value.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}
impl<'de, T> Deserialize<'de> for Maybe<T>
where
    T: Deserialize<'de> + Serialize
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let opt = Some(T::deserialize(deserializer)?);
        Ok(Maybe(opt))
    }
}

impl<T: Serialize> From<Option<T>> for Maybe<T> {
    fn from(opt: Option<T>) -> Self {
        Maybe(opt)
    }
}
impl<T: Serialize> From<Maybe<T>> for Option<T> {
    fn from(maybe: Maybe<T>) -> Self {
        maybe.0
    }
}

impl<T: Serialize + Debug> Debug for Maybe<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Maybe").field(&self.0).finish()
    }
}
impl<T: Serialize + Clone> Clone for Maybe<T> {
    fn clone(&self) -> Self {
        Maybe(self.0.clone())
    }
}
impl<T: Serialize + Copy> Copy for Maybe<T> {}