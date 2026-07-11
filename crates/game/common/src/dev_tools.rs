use bevy::prelude::*;

pub use debug_option_derive::*;

pub trait DebugOption: Component {
    fn get(&self) -> bool;
    fn set(&mut self, value: bool);
}

pub trait DebugOptionResource: Resource + DebugOption {}
impl<T> DebugOptionResource for T where T: Resource + DebugOption {}

pub trait DebugOptionCategoryElement {}
impl<T> DebugOptionCategoryElement for T where T: DebugOption {}

pub trait DebugOptionCategory: DebugOptionCategoryElement {}