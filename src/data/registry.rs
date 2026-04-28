use std::collections::{HashMap, HashSet};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use crate::data::{ResourceLocation, ResourceType};

#[derive(SystemParam)]
pub struct SystemRegistry<'w, T: ResourceType> {
    registry: Res<'w, ResourceRegistry<T>>,
    assets: Res<'w, Assets<<T as ResourceType>::AssetType>>,
}
impl<T: ResourceType> SystemRegistry<'_, T> {
    pub fn get_handle(&self, id: ResourceLocation<T>) -> Option<Handle<T::AssetType>> {
        self.registry.registry.get(&id).cloned()
    }

    pub fn get_asset(&self, id: ResourceLocation<T>) -> Option<&T::AssetType> {
        let handle = self.get_handle(id);
        handle.and_then(|handle| self.get_asset_from_handle(handle))
    }

    pub fn get_asset_from_handle(&self, handle: Handle<T::AssetType>) -> Option<&T::AssetType> {
        self.assets.get(handle.id())
    }

    pub fn registry(&self) -> &ResourceRegistry<T> {
        &self.registry
    }
}

#[derive(SystemParam)]
pub struct SystemRegistryMut<'w, T: ResourceType> {
    registry: ResMut<'w, ResourceRegistry<T>>,
    assets: ResMut<'w, Assets<<T as ResourceType>::AssetType>>,
}
impl<T: ResourceType> SystemRegistryMut<'_, T> {
    pub fn get_handle(&self, id: ResourceLocation<T>) -> Option<Handle<T::AssetType>> {
        self.registry.registry.get(&id).cloned()
    }

    pub fn get_asset(&self, id: ResourceLocation<T>) -> Option<&T::AssetType> {
        let handle = self.get_handle(id);
        handle.and_then(|handle| self.get_asset_from_handle(handle))
    }

    pub fn get_asset_mut(&mut self, id: ResourceLocation<T>) -> Option<&mut T::AssetType> {
        let handle = self.get_handle(id);
        handle.and_then(|handle| self.get_asset_from_handle_mut(handle))
    }

    pub fn get_asset_from_handle(&self, handle: Handle<T::AssetType>) -> Option<&T::AssetType> {
        self.assets.get(handle.id())
    }

    pub fn get_asset_from_handle_mut(&mut self, handle: Handle<T::AssetType>) -> Option<&mut T::AssetType> {
        self.assets.get_mut(handle.id())
    }

    pub fn registry(&self) -> &ResourceRegistry<T> {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut ResourceRegistry<T> {
        &mut self.registry
    }
}

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

    pub fn iter(&self) -> impl Iterator<Item = (&ResourceLocation<T>, &Handle<T::AssetType>)> {
        self.registry.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ResourceLocation<T>, &mut Handle<T::AssetType>)> {
        self.registry.iter_mut()
    }
}

impl<T: ResourceType> IntoIterator for ResourceRegistry<T> {
    type Item = (ResourceLocation<T>, Handle<T::AssetType>);
    type IntoIter = std::collections::hash_map::IntoIter<ResourceLocation<T>, Handle<T::AssetType>>;

    fn into_iter(self) -> Self::IntoIter {
        self.registry.into_iter()
    }
}