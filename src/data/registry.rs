use std::collections::HashMap;
use bevy::prelude::*;
use crate::data::{ResourceLocation, ResourceType};

/// Maps resource locations to bevy asset handles
#[derive(Default, Debug, Resource)]
pub struct ResourceRegistry<T: ResourceType, A: Asset> {
    registry: HashMap<ResourceLocation<T>, Handle<A>>
}

impl<T: ResourceType, A: Asset> ResourceRegistry<T, A> {
    pub fn insert(&mut self, loc: ResourceLocation<T>, handle: Handle<A>) {
        self.registry.insert(loc, handle);
    }

    pub fn get(&self, loc: &ResourceLocation<T>) -> Option<&Handle<A>> {
        self.registry.get(loc)
    }

    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }

    pub fn len(&self) -> usize {
        self.registry.len()
    }
}