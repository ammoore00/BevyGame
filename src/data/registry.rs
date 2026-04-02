use std::collections::{HashMap, HashSet};
use bevy::prelude::*;
use crate::data::{ResourceLocation, ResourceType};

/// Maps resource locations to bevy asset handles
#[derive(Debug, Resource)]
pub struct ResourceRegistry<T: ResourceType, A: Asset> {
    /// Stores the mapping between resource locations and asset handles
    registry: HashMap<ResourceLocation<T>, Handle<A>>,
    /// Stores a list of desired resource locations
    manifest: HashSet<ResourceLocation<T>>,
}

impl<T: ResourceType, A: Asset> Default for ResourceRegistry<T, A> {
    fn default() -> Self {
        Self {
            registry: Default::default(),
            manifest: Default::default(),
        }
    }
}

impl<T: ResourceType, A: Asset> ResourceRegistry<T, A> {
    /// Adds a resource location to the manifest
    pub fn insert_manifest(&mut self, loc: ResourceLocation<T>) {
        self.manifest.insert(loc);
    }

    /// Adds a list of resource locations to the manifest
    pub fn extend_manifest(&mut self, manifest: Vec<ResourceLocation<T>>) {
        self.manifest.extend(manifest);
    }

    /// Inserts a resource location and asset handle into the registry after the asset has been loaded
    pub fn register_asset(&mut self, loc: ResourceLocation<T>, handle: Handle<A>) {
        self.registry.insert(loc, handle);
    }

    /// Checks if the provided resource location is present in the registry
    pub fn asset_loaded(&self, loc: &ResourceLocation<T>) -> Result<bool, ResourceRegistryError> {
        if !self.manifest.contains(loc) {
            return Err(ResourceRegistryError::ResourceLocationNotInManifest(loc.to_string()));
        }
        Ok(self.registry.contains_key(loc))
    }

    /// Checks if all assets in the manifest have been loaded
    pub fn all_loaded(&self) -> bool {
        self.manifest.iter().all(|e| self.registry.contains_key(e))
    }

    pub fn get(&self, loc: &ResourceLocation<T>) -> Option<&Handle<A>> {
        self.registry.get(loc)
    }
    
    pub fn manifest(&self) -> &HashSet<ResourceLocation<T>> {
        &self.manifest
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceRegistryError {
    #[error("Resource location {0} is not in the manifest")]
    ResourceLocationNotInManifest(String)
}