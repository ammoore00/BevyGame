use std::collections::HashMap;
use std::sync::Arc;
use bevy::prelude::*;
use crate::data::registry::ResourceRegistry;
use crate::data::ResourceType;
use crate::StartupSystems;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, load_assets.in_set(StartupSystems::LoadAssets));
}

fn load_assets(world: &mut World) {
    let loader = world.resource::<AssetLoader>();
    let jobs = loader.jobs.clone();
    jobs.iter().for_each(|job| job.load(world).expect("Failed to load assets"));
}

/// Resource which holds a list of jobs to load assets
/// The jobs retrieve the `ResourceRegistry` from the World,
/// load each asset, then insert them into the registry
#[derive(Default, Resource)]
pub struct AssetLoader {
    jobs: Vec<Arc<dyn RegistryLoader>>,
}
impl AssetLoader {
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

        if !world.contains_resource::<AssetLoader>() {
            world.insert_resource(AssetLoader::new());
        }

        let mut asset_loader = world.resource_mut::<AssetLoader>();
        asset_loader.add_job::<T, A>();
    }
}

trait RegistryLoader: Send + Sync + 'static {
    fn load(&self, world: &mut World) -> Result<(), LoaderError>;
}

#[derive(Debug)]
struct LoaderJob<T: ResourceType, A: Asset> {
    phantom_data: std::marker::PhantomData<(T, A)>,
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

#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
}