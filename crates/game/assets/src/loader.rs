use crate::state::{AssetLoadState, AssetSystems};
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::erased_serde::__private::serde::Deserializer;
use data::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, (load_assets.in_set(AssetSystems::LoadAssets),));

    app.add_systems(
        Update,
        (
            finish_load_state.run_if(in_state(AssetLoadState::Loading)),
            finish_resolving_state.run_if(in_state(AssetLoadState::Resolving)),
        )
    );

    app.add_systems(OnEnter(AssetLoadState::Resolving), visit_assets);
}

fn load_assets(world: &mut World) {
    info!("Loading assets...");

    let loader = world.resource::<GameAssetLoader>();
    info!("Jobs: {}", loader.job_list_display());

    let jobs = loader.loader_jobs.clone();
    jobs.iter()
        .for_each(|(job, _)| job.load(world).expect("Failed to load resource"));
}

fn visit_assets(world: &mut World) {
    info!("Processing assets...");
    let loader = world.resource_mut::<GameAssetLoader>();
    let mut jobs = loader.loader_jobs.clone();
    jobs.iter_mut().for_each(|(job, visited)| {
        job.visit(world);
        *visited.write().unwrap() = true;
    });
}

fn finish_load_state(world: &mut World) {
    let loader = world.resource::<GameAssetLoader>();

    if loader.loader_jobs.iter().all(|(job, _)| job.is_loaded(world)) {
        info!("Assets loaded");
        let mut next_state = world.resource_mut::<NextState<AssetLoadState>>();
        next_state.set(AssetLoadState::Resolving);
    }
}

fn finish_resolving_state(
    loader: Res<GameAssetLoader>,
    mut next_state: ResMut<NextState<AssetLoadState>>,
) {
    if loader.loader_jobs.iter().all(|(_, visited)| *visited.read().unwrap()) {
        info!("Assets processed");
        next_state.set(AssetLoadState::Done);
    }
}

/// Resource which holds a list of jobs to load resource
/// The jobs retrieve the `ResourceRegistry` from the World,
/// load each asset, then insert them into the registry
#[derive(Default, Resource)]
pub struct GameAssetLoader {
    loader_jobs: Vec<(Arc<dyn RegistryLoader>, Arc<RwLock<bool>>)>,
}
impl GameAssetLoader {
    pub fn new() -> Self {
        Self {
            loader_jobs: Vec::new(),
        }
    }

    pub fn add_loader_job<T: ResourceKind>(&mut self) {
        self.loader_jobs.push((Arc::new(LoaderJob::<T>::default()), Arc::new(RwLock::new(false))));
    }

    pub fn job_list_display(&self) -> String {
        self.loader_jobs
            .iter()
            .map(|(job, _)| job.name().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub trait LoaderJobManager {
    /// Adds a job to the asset loader which will load all resources in the registry
    fn add_resource_registry<T: ResourceKind>(&mut self) -> &mut Self;
    /// Adds a job to the asset loader with a pre-filled manifest
    fn add_registry_with_manifest<T: ResourceKind>(
        &mut self,
        manifest: Vec<ResourceLocation<T>>,
    ) -> &mut Self;
    /// Adds a job to the asset loader which will discover all resources in the registry automatically
    fn add_registry_with_discovery<T: ResourceKind>(&mut self) -> &mut Self;
}

impl LoaderJobManager for App {
    fn add_resource_registry<T: ResourceKind>(&mut self) -> &mut Self {
        let world = self.world_mut();

        if world.contains_resource::<ResourceRegistry<T>>() {
            panic!("Cannot contain duplicate registries!")
        }
        world.insert_resource(ResourceRegistry::<T>::default());

        if !world.contains_resource::<GameAssetLoader>() {
            world.insert_resource(GameAssetLoader::new());
        }

        let mut asset_loader = world.resource_mut::<GameAssetLoader>();
        asset_loader.add_loader_job::<T>();

        self
    }

    fn add_registry_with_manifest<T: ResourceKind>(
        &mut self,
        manifest: Vec<ResourceLocation<T>>,
    ) -> &mut Self {
        self.add_resource_registry::<T>();
        let world = self.world_mut();
        let mut registry = world.resource_mut::<ResourceRegistry<T>>();
        registry.extend_manifest(manifest);

        self
    }

    // TODO: Eventually I want to be able to load resource from multiple places (e.g. mod files)
    //       This will require a way to check all places, not just the normal resource folder
    fn add_registry_with_discovery<T: ResourceKind>(&mut self) -> &mut Self {
        // Find all namespaces currently available
        let Ok(namespaces) = std::fs::read_dir("./assets/") else {
            error!("Failed to read asset directory!");
            return self;
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
                .filter(|path| {
                    path.strip_prefix(namespace)
                        .unwrap()
                        .starts_with(T::ROOT_DIR)
                })
                .filter_map(|path| ResourceLocation::from_path(path).ok())
                .for_each(|location| manifest.push(location));
        }

        info!("Found {} assets in {}", manifest.len(), T::ROOT_DIR);

        self.add_registry_with_manifest::<T>(manifest);

        self
    }
}

trait RegistryLoader: Send + Sync + 'static {
    fn load(&self, world: &mut World) -> Result<(), LoaderError>;
    fn is_loaded(&self, world: &World) -> bool;
    fn visit(&self, world: &mut World);
    fn name(&self) -> &str;
}

#[derive(Debug)]
struct LoaderJob<T: ResourceKind> {
    phantom_data: PhantomData<T>,
}
impl<T: ResourceKind> Default for LoaderJob<T> {
    fn default() -> Self {
        Self {
            phantom_data: Default::default(),
        }
    }
}
impl<T: ResourceKind> RegistryLoader for LoaderJob<T> {
    /// Iterate through all registered resource for the associated registry and loads them
    fn load(&self, world: &mut World) -> Result<(), LoaderError> {
        info!("--- Loading assets in {} ---", self.name());

        let asset_server = world.resource::<AssetServer>();

        let mut assets = HashMap::new();

        let registry = world.resource::<ResourceRegistry<T>>();
        let manifest = registry.manifest();
        manifest.iter().for_each(|loc| {
            info!("Loading {}", loc.as_path().to_str().unwrap());
            let path = loc.as_path();
            let asset = asset_server.load::<T::AssetKind>(path);
            assets.insert(loc.clone(), asset);
        });

        let mut registry = world.resource_mut::<ResourceRegistry<T>>();
        assets
            .into_iter()
            .for_each(|(loc, asset)| {
                registry.register_asset(loc, asset);
            });

        Ok(())
    }

    fn is_loaded(&self, world: &World) -> bool {
        let asset_server = world.resource::<AssetServer>();
        let registry = world.resource::<ResourceRegistry<T>>();

        registry
            .iter()
            .all(|(_, handle)| asset_server.is_loaded_with_dependencies(handle))
    }

    fn visit(&self, world: &mut World) {
        let registry = world.resource::<ResourceRegistry<T>>();
        let assets = world.resource::<Assets<T::AssetKind>>();

        let registry = registry
            .iter()
            .map(|(loc, handle)| (loc.clone(), assets.get(handle).unwrap().clone()))
            .collect::<Vec<_>>();

        let mut new_assets = HashMap::new();

        for (loc, asset) in registry {
            info!("Visiting {}", loc.as_path().to_str().unwrap());

            match T::visit(loc.clone(), asset.clone(), world) {
                Ok(asset) => {
                    new_assets.insert(loc, asset);
                }
                Err(err) => {
                    // TODO: Doing nothing to correct the error will eventually lead to a panic,
                    //  so handle this more properly
                    error!("Error visiting asset: {}", err);
                }
            }
        }

        let registry = world.resource::<ResourceRegistry<T>>();
        let new_assets = new_assets
            .into_iter()
            .map(|(loc, asset)| (registry.get(&loc).unwrap().clone(), asset))
            .collect::<HashMap<_, _>>();

        let mut assets = world.resource_mut::<Assets<T::AssetKind>>();

        for (handle, asset) in new_assets {
            let mut asset_ptr = assets.get_mut(&handle).unwrap();
            *asset_ptr = asset;
        }
    }

    fn name(&self) -> &str {
        T::ROOT_DIR
    }
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {}

/// Marker trait for types which can be loaded from a file
pub trait RonCodec<AssetType>: TryInto<AssetType> + TypePath + Send + Sync + 'static
where
    <Self as TryInto<AssetType>>::Error: Debug + Send + Sync + 'static,
{
}

impl<T, AssetType> RonCodec<AssetType> for T
where
    T: TryInto<AssetType> + TypePath + Send + Sync + 'static,
    <T as TryInto<AssetType>>::Error: Debug + Send + Sync + 'static,
{
}

#[derive(TypePath)]
pub struct RonAssetLoader<Codec, AssetType>
where
    Codec: DeserializeOwned + RonCodec<AssetType>,
    <Codec as TryInto<AssetType>>::Error: Debug + Send + Sync + 'static,
    AssetType: Asset + Send + Sync + 'static,
{
    phantom_data: PhantomData<(Codec, AssetType)>,
}
impl<Codec, AssetType> Default for RonAssetLoader<Codec, AssetType>
where
    Codec: DeserializeOwned + RonCodec<AssetType>,
    <Codec as TryInto<AssetType>>::Error: Debug + Send + Sync + 'static,
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
    Codec: DeserializeOwned + RonCodec<AssetType>,
    <Codec as TryInto<AssetType>>::Error: Debug + Send + Sync + 'static,
    AssetType: Asset + Send + Sync + 'static,
{
    type Asset = AssetType;
    type Settings = ();
    type Error = RonLoaderError<Codec, AssetType>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let codec = ron::de::from_bytes::<Codec>(&bytes)?;
        codec.try_into().map_err(RonLoaderError::from_codec_err)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(thiserror::Error)]
pub enum RonLoaderError<Codec, AssetType>
where
    Codec: DeserializeOwned + RonCodec<AssetType>,
    <Codec as TryInto<AssetType>>::Error: Debug + Send + Sync + 'static,
    AssetType: Asset + Send + Sync + 'static,
{
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RON parse error: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("Codec error")]
    Codec(<Codec as TryInto<AssetType>>::Error),
}
impl<Codec, AssetType> RonLoaderError<Codec, AssetType>
where
    Codec: DeserializeOwned + RonCodec<AssetType>,
    <Codec as TryInto<AssetType>>::Error: Debug + Send + Sync + 'static,
    AssetType: Asset + Send + Sync + 'static,
{
    fn from_codec_err(value: <Codec as TryInto<AssetType>>::Error) -> Self {
        Self::Codec(value)
    }
}
// Manual implementation of debug trait because derive would require AssetType and Codec
// to implement Debug, when nothing in the implementation actually requires that
impl<Codec, AssetType> Debug for RonLoaderError<Codec, AssetType>
where
    Codec: DeserializeOwned + RonCodec<AssetType>,
    <Codec as TryInto<AssetType>>::Error: Debug + Send + Sync + 'static,
    AssetType: Asset + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Ron(err) => err.fmt(f),
            Self::Codec(err) => err.fmt(f),
        }
    }
}

/// Wrapper around Option<T> to be used as an optional value within a codec
/// Used instead of Option to avoid Option enum values being included in
/// output files
pub struct Maybe<T: Serialize>(pub Option<T>);
impl<T: Serialize> Maybe<T> {
    pub fn into_inner(self) -> Option<T> {
        self.0
    }
}
impl<T: Serialize> Default for Maybe<T> {
    fn default() -> Self {
        Self(None)
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
    T: Deserialize<'de> + Serialize,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let result = T::deserialize(deserializer);
        let opt = Some(result?);
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

/// A type which allows either inline data, or a reference to an asset
#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
#[serde(
    untagged,
    bound(
        serialize = "ResourceLocation<T>: Serialize, Codec: Serialize",
        deserialize = "ResourceLocation<T>: Deserialize<'de>, Codec: Deserialize<'de>"
    )
)]
pub enum _InlineOrResourceLocation<T, Codec>
where
    T: ResourceKind,
    Codec: RonCodec<T::AssetKind>,
    <Codec as TryInto<T::AssetKind>>::Error: Debug + Send + Sync + 'static,
{
    Inline(Codec),
    ResourceLocation(ResourceLocation<T>),
}
impl<T, Codec> _InlineOrResourceLocation<T, Codec>
where
    T: ResourceKind,
    Codec: RonCodec<T::AssetKind>,
    <Codec as TryInto<T::AssetKind>>::Error: Debug + Send + Sync + 'static,
{
    pub fn _resolve(self, registry: &SystemRegistry<T>) -> Option<T::AssetKind> {
        match self {
            _InlineOrResourceLocation::Inline(codec) => codec.try_into().ok(),
            _InlineOrResourceLocation::ResourceLocation(location) => {
                registry.get_asset(&location).cloned()
            }
        }
    }
}
