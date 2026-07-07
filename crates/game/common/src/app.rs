use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // Main game loop systems
    app.configure_sets(
        Update,
        (
            AppSystems::TickTimers,
            AppSystems::RecordInput,
            AppSystems::Update,
            AppSystems::Respond,
        )
            .chain(),
    );

    // Set up the `Pause` state.
    app.init_state::<Pause>();
    app.configure_sets(Update, PausableSystems.run_if(in_state(Pause::Unpaused)));
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
    /// Respond to changes in update
    Respond,
}

/// Whether the game is paused.
#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Pause {
    /// The game is not paused
    #[default]
    Unpaused,
    /// The game was paused by the pause menu
    Paused,
    /// The game was paused by something else happening, storing the previous state
    ForcePaused(Box<Pause>),
}

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PausableSystems;