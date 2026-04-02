use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use serde::de::DeserializeOwned;
use crate::data::registry::ResourceRegistry;
use crate::data::ResourceType;
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

    pub fn add_job<T: ResourceType, A: Asset>(&mut self) {
        self.jobs.push(Arc::new(LoaderJob::<T, A>::default()));
    }
}

pub trait LoaderJobManager {
    fn add_resource_registry<T: ResourceType, A: Asset>(&mut self);
}

impl LoaderJobManager for App {
    /// Adds a job to the asset loader which will load all assets in the registry
    fn add_resource_registry<T: ResourceType, A: Asset>(&mut self) {
        let world = self.world_mut();
        world.insert_resource(ResourceRegistry::<T, A>::default());

        if !world.contains_resource::<GameAssetLoader>() {
            world.insert_resource(GameAssetLoader::new());
        }

        let mut asset_loader = world.resource_mut::<GameAssetLoader>();
        asset_loader.add_job::<T, A>();
    }
}

trait RegistryLoader: Send + Sync + 'static {
    fn load(&self, world: &mut World) -> Result<(), LoaderError>;
}

#[derive(Debug)]
struct LoaderJob<T: ResourceType, A: Asset> {
    phantom_data: PhantomData<(T, A)>,
}
impl<T: ResourceType, A: Asset> Default for LoaderJob<T, A> {
    fn default() -> Self {
        Self {
            phantom_data: Default::default()
        }
    }
}
impl<T: ResourceType, A: Asset> RegistryLoader for LoaderJob<T, A> {
    /// Iterate through all registered assets for the associated registry and loads them
    fn load(&self, world: &mut World) -> Result<(), LoaderError> {
        let asset_server = world.resource::<AssetServer>();

        let mut assets = HashMap::new();

        let registry = world.resource::<ResourceRegistry<T, A>>();
        let manifest = registry.manifest();
        manifest.iter().for_each(|loc| {
            let path = loc.as_path();
            let asset = asset_server.load::<A>(path);
            assets.insert(loc.clone(), asset);
        });

        let mut registry = world.resource_mut::<ResourceRegistry<T, A>>();
        assets.into_iter().for_each(|(loc, asset)| registry.register_asset(loc, asset));

        Ok(())
    }
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
}

pub struct RonAssetLoader<Codec, AssetType>
where
    Codec: DeserializeOwned + Into<AssetType> + Send + Sync + 'static,
    AssetType: Asset + Send + Sync + 'static,
{
    phantom_data: PhantomData<(Codec, AssetType)>,
}
impl<Codec, AssetType> Default for RonAssetLoader<Codec, AssetType>
where
    Codec: DeserializeOwned + Into<AssetType> + Send + Sync + 'static,
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
    Codec: DeserializeOwned + Into<AssetType> + Send + Sync + 'static,
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