use bevy::dev_tools::states::log_transitions;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use crate::dev_tools::{spawn_checkbox_row, spawn_debug_category, DebugMenuEvent, DebugSetting, LoggingScreenStates};
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<LoggingScreenStates>();

    app.add_systems(
        Update,
        (
            log_transitions::<Screen>.run_if(in_state(LoggingScreenStates(true))),
        )
    );

    app.add_observer(on_ui_debug);
    app.add_observer(on_log_screen_state);
}

pub(super) fn spawn_debug(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    log_screen_states: Res<State<LoggingScreenStates>>,
    ui_debug_options: Res<UiDebugOptions>,
) {
    spawn_debug_category(
        parent,
        "UI",
        [
            ui_debug_options.enabled,
        ].as_slice().into(),
        false,
        |parent| {
            spawn_checkbox_row(
                parent,
                "UI Debug Overlay",
                ui_debug_options.enabled,
                DebugUi,
            );
        },
    );

    spawn_debug_category(
        parent,
        "States",
        [
            log_screen_states.0,
        ].as_slice().into(),
        false,
        |parent| {
            spawn_checkbox_row(
                parent,
                "Log Screen State Transitions",
                log_screen_states.0,
                LogScreenStateTransitions,
            );
        },
    );
}

#[derive(Component, Debug, Clone)]
struct LogScreenStateTransitions;
impl DebugSetting for LogScreenStateTransitions {}

#[derive(Component, Debug, Clone)]
struct DebugUi;
impl DebugSetting for DebugUi {}

macro_rules! debug_menu_event {
    (
        $marker:ty,
        fn $fn_name:ident(
            $event:ident: $event_ty:ty,
            $($args:tt)*
        ) $content:block
    ) => {
        fn $fn_name(
            $event: $event_ty,
            __entity_query: Query<Entity, With<$marker>>,
            $($args)*
        ) {
            let $event: On<DebugMenuEvent> = $event;
            match __entity_query.single() {
                Ok(entity) => {
                    if $event.entity == entity {
                        $content
                    }
                }
                Err(err) => {
                    error!("Failed to obtain entity: {}", err);
                }
            }
        }
    };
}

debug_menu_event!(
    DebugUi,
    fn on_ui_debug(
        event: On<DebugMenuEvent>,
        mut ui_debug_options: ResMut<UiDebugOptions>,
    ) {
        ui_debug_options.toggle();
        info!("UI Debug toggled: {}", ui_debug_options.enabled);
    }
);

debug_menu_event!(
    LogScreenStateTransitions,
    fn on_log_screen_state(
        event: On<DebugMenuEvent>,
        log_state: ResMut<State<LoggingScreenStates>>,
        mut next_state: ResMut<NextState<LoggingScreenStates>>,
    ) {
        next_state.set(LoggingScreenStates(!log_state.0));
        info!("Logging for screen state transitions toggled: {}", log_state.0);
    }
);