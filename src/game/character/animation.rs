use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    
}

#[derive(Component, Debug, Clone, Reflect)]
pub enum CharacterAnimation {
    Static,
    Dynamic,
}