use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::data::loc::ResourceLocation;
use crate::data::resource::ResourceKind;
// TODO: Change registry to dynamically register loaded assets instead of loading them all at once
//       to support new BSN inline asset definitions

/// Maps resource locations to bevy asset handles
#[derive(Debug, Resource)]
pub struct ResourceRegistry<T: ResourceKind> {
    /// Stores the mapping between resource locations and asset handles
    registry: HashMap<ResourceLocation<T>, Handle<T::AssetKind>>,
    /// Stores a list of desired resource locations
    manifest: HashSet<ResourceLocation<T>>,
}
impl<T: ResourceKind> ResourceRegistry<T> {
    /// Adds a resource location to the manifest
    pub fn insert_manifest(&mut self, loc: ResourceLocation<T>) {
        self.manifest.insert(loc);
    }

    /// Adds a list of resource locations to the manifest
    pub fn extend_manifest(&mut self, manifest: Vec<ResourceLocation<T>>) {
        self.manifest.extend(manifest);
    }

    /// Inserts a resource location and asset handle into the registry after the asset has been loaded
    pub fn register_asset(&mut self, loc: ResourceLocation<T>, handle: Handle<T::AssetKind>) {
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

    pub fn get(&self, loc: &ResourceLocation<T>) -> Option<&Handle<T::AssetKind>> {
        self.registry.get(loc)
    }

    pub fn manifest(&self) -> &HashSet<ResourceLocation<T>> {
        &self.manifest
    }

    pub fn handles(&self) -> impl Iterator<Item = &Handle<T::AssetKind>> {
        self.registry.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ResourceLocation<T>, &Handle<T::AssetKind>)> {
        self.registry.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ResourceLocation<T>, &mut Handle<T::AssetKind>)> {
        self.registry.iter_mut()
    }
}
impl<T: ResourceKind> Default for ResourceRegistry<T> {
    fn default() -> Self {
        Self {
            registry: Default::default(),
            manifest: Default::default(),
        }
    }
}
impl<T: ResourceKind> IntoIterator for ResourceRegistry<T> {
    type Item = (ResourceLocation<T>, Handle<T::AssetKind>);
    type IntoIter = std::collections::hash_map::IntoIter<ResourceLocation<T>, Handle<T::AssetKind>>;

    fn into_iter(self) -> Self::IntoIter {
        self.registry.into_iter()
    }
}

macro_rules! registry_impl {
    (
        impl<$type_param:ident : $bound:path> $system_param:ident {
            $handle_fn:ident,
            $asset_fn:ident,
            $asset_from_handle_fn:ident,
        }
    ) => {
        impl<T: $bound> $system_param<'_, $type_param> {
            #[allow(unused)]
            pub fn $handle_fn(&self, id: &ResourceLocation<$type_param>) -> Option<Handle<$type_param::AssetKind>> {
                self.registry.registry.get(&id).cloned()
            }

            #[allow(unused)]
            pub fn $asset_fn(&self, id: &ResourceLocation<$type_param>) -> Option<&$type_param::AssetKind> {
                let handle = self.$handle_fn(id);
                handle.and_then(|handle| self.$asset_from_handle_fn(handle))
            }

            #[allow(unused)]
            pub fn $asset_from_handle_fn(&self, handle: Handle<$type_param::AssetKind>) -> Option<&$type_param::AssetKind> {
                self.assets.get(handle.id())
            }
        }
    };
}

#[derive(SystemParam, getset::Getters)]
pub struct SystemRegistry<'w, T: ResourceKind> {
    #[getset(get = "pub")]
    registry: Res<'w, ResourceRegistry<T>>,
    #[getset(get = "pub")]
    assets: Res<'w, Assets<<T as ResourceKind>::AssetKind>>,
}
registry_impl!(
    impl<T: ResourceKind> SystemRegistry {
        get_handle,
        get_asset,
        get_asset_from_handle,
    }
);

#[derive(SystemParam, getset::Getters, getset::MutGetters)]
pub struct SystemRegistryMut<'w, T: ResourceKind> {
    #[getset(get = "pub", get_mut = "pub")]
    registry: ResMut<'w, ResourceRegistry<T>>,
    #[getset(get = "pub", get_mut = "pub")]
    assets: ResMut<'w, Assets<<T as ResourceKind>::AssetKind>>,
}
registry_impl!(
    impl<T: ResourceKind> SystemRegistryMut {
        get_handle,
        get_asset,
        get_asset_from_handle,
    }
);
impl<T: ResourceKind> SystemRegistryMut<'_, T> {
    pub fn split(
        &mut self,
    ) -> (
        &mut ResourceRegistry<T>,
        &mut Assets<T::AssetKind>,
    ) {
        (&mut self.registry, &mut self.assets)
    }
}