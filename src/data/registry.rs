use std::collections::{HashMap, HashSet};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use crate::data::{ResourceLocation, ResourceType};

#[derive(SystemParam)]
pub struct SystemRegistry<'w, T: ResourceType>(Res<'w, ResourceRegistry<T>>);

/// Maps resource locations to bevy asset handles
#[derive(Debug, Resource)]
pub struct ResourceRegistry<T: ResourceType> {
    /// Stores the mapping between resource locations and asset handles
    registry: HashMap<ResourceLocation<T>, Handle<T::AssetType>>,
    /// Stores a list of desired resource locations
    manifest: HashSet<ResourceLocation<T>>,
}

impl<T: ResourceType> Default for ResourceRegistry<T> {
    fn default() -> Self {
        Self {
            registry: Default::default(),
            manifest: Default::default(),
        }
    }
}

impl<T: ResourceType> ResourceRegistry<T> {
    /// Adds a resource location to the manifest
    pub fn insert_manifest(&mut self, loc: ResourceLocation<T>) {
        self.manifest.insert(loc);
    }

    /// Adds a list of resource locations to the manifest
    pub fn extend_manifest(&mut self, manifest: Vec<ResourceLocation<T>>) {
        self.manifest.extend(manifest);
    }

    /// Inserts a resource location and asset handle into the registry after the asset has been loaded
    pub fn register_asset(&mut self, loc: ResourceLocation<T>, handle: Handle<T::AssetType>) {
        self.registry.insert(loc, handle);
    }

    /// Checks if the provided resource location is present in the registry
    pub fn is_loaded(&self, loc: &ResourceLocation<T>) -> bool {
        self.registry.contains_key(loc)
    }

    /// Checks if the provided resource location is present in the manifest
    pub fn is_requested(&self, loc: &ResourceLocation<T>) -> bool {
        self.manifest.contains(loc)
    }

    /// Checks if all assets in the manifest have been loaded
    pub fn all_loaded(&self) -> bool {
        self.manifest.iter().all(|e| self.registry.contains_key(e))
    }

    pub fn get(&self, loc: &ResourceLocation<T>) -> Option<&Handle<T::AssetType>> {
        self.registry.get(loc)
    }

    pub fn manifest(&self) -> &HashSet<ResourceLocation<T>> {
        &self.manifest
    }
}