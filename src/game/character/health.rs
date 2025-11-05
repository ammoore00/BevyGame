use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {}

#[derive(Component, Asset, Clone, Reflect)]
pub struct Health {
    pub max: usize,
    pub current: usize,
}

impl Health {
    pub fn new(max: usize) -> Self {
        Self { max, current: max }
    }
}
