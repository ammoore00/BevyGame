use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>();
    app.configure_sets(Update, GameplaySystems.run_if(in_state(GameState::Gameplay)));
}

// TODO: Should this be in this crate?
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Menu,
    Gameplay,
}

/// Systems which should run during gameplay
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GameplaySystems;