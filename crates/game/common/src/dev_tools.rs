use crate::marker;
use bevy::prelude::*;

pub use debug_option_derive::*;

marker!(pub DebugEntry);
marker!(pub DebugCategory);

pub trait DebugOption: Component + Clone + Default {
    fn get(&self) -> bool;
    fn set(&mut self, value: bool);
}

pub trait DebugOptionResource: Resource + DebugOption {}
impl<T> DebugOptionResource for T where T: Resource + DebugOption {}