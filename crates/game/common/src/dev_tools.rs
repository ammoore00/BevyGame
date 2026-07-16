use crate::marker;
use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use std::fmt::Debug;

pub use debug_option_derive::*;

marker!(pub DebugEntry);
marker!(pub DebugCategory);

pub trait DebugOption: Component + Clone + Default + Debug {
    type Res: DebugState<Mutability = Mutable>;

    fn get(&self) -> bool;
    fn set(&mut self, value: bool);
}

pub trait DebugState: Resource + Clone + Default {
    fn get(&self) -> bool;
    fn set(&mut self, value: bool);
}