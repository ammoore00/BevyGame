use crate::dev_tools::debug_menu::{spawn_checkbox_row, spawn_debug_category, DebugMenuEvent, DebugSetting};
use crate::screens::Screen;
use bevy::dev_tools::states::log_transitions;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use paste::paste;

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
                    if $event.entity() == entity {
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

macro_rules! debug_setting {
    ($setting:ident, $event_fn:ident, $label:literal, $desc:literal) => {
        paste! {
            #[derive(Component, Debug, Clone)]
            struct $setting;
            impl DebugSetting for $setting {}

            #[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
            pub(super) struct [<$setting State>](pub bool);

            debug_menu_event!(
                $setting,
                fn [<on_ $event_fn>] (
                    event: On<DebugMenuEvent>,
                    prev_state: Res<State<[<$setting State>]>>,
                    mut next_state: ResMut<NextState<[<$setting State>]>>,
                ) {
                    next_state.set([<$setting State>](!prev_state.0));
                    info!("{} toggled: {}", $desc, !prev_state.0);
                }
            );

            fn [<spawn_ $event_fn _checkbox>](
                parent: &mut ChildSpawnerCommands,
                initial_state: bool,
            ) {
                spawn_checkbox_row(
                    parent,
                    $label,
                    initial_state,
                    $setting,
                );
            }
        }
    };
}

pub(in crate::dev_tools) fn plugin(app: &mut App) {
    app.init_state::<LoggingScreenStateTransitionsState>();

    app.init_state::<RenderNavMapNodesState>();
    app.init_state::<RenderNavMapEdgesState>();
    app.init_state::<RenderNPCPathsState>();

    app.init_state::<RenderPhysicsEntitiesState>();
    app.init_state::<RenderPhysicsTilesState>();

    app.add_systems(
        Update,
        (
            log_transitions::<Screen>.run_if(in_state(LoggingScreenStateTransitionsState(true))),
        )
    );

    app.add_observer(on_ui_debug);

    app.add_observer(on_log_screen_state);

    app.add_observer(on_render_nav_map_nodes);
    app.add_observer(on_render_nav_map_edges);
    app.add_observer(on_render_npc_paths);

    app.add_observer(on_physics_render_entities);
    app.add_observer(on_physics_render_tiles);
}

debug_setting!(LoggingScreenStateTransitions, log_screen_state, "Log Screen State Transitions", "Logging for screen state transitions");

debug_setting!(RenderNavMapNodes, render_nav_map_nodes, "Render Nodes", "Nav map nodes");
debug_setting!(RenderNavMapEdges, render_nav_map_edges, "Render Edges", "Nav map edges");
debug_setting!(RenderNPCPaths, render_npc_paths, "Render NPC Paths", "NPC Paths");

debug_setting!(RenderPhysicsEntities, physics_render_entities, "Render Entity Collision", "Entity physics renderer");
debug_setting!(RenderPhysicsTiles, physics_render_tiles, "Render Tile Collision", "Tile physics renderer transitions");

#[derive(SystemParam)]
pub(crate) struct DebugOptionState<'w> {
    // TODO: Add new ui debug options from bevy 0.19
    render_ui_debug: Res<'w, GlobalUiDebugOptions>,

    log_screen_transitions: Res<'w, State<LoggingScreenStateTransitionsState>>,

    render_nav_map_nodes: Res<'w, State<RenderNavMapNodesState>>,
    render_nav_map_edges: Res<'w, State<RenderNavMapEdgesState>>,
    render_npc_paths: Res<'w, State<RenderNPCPathsState>>,

    render_physics_entity: Res<'w, State<RenderPhysicsEntitiesState>>,
    render_physics_tile: Res<'w, State<RenderPhysicsTilesState>>,
}

pub(super) fn spawn_debug(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    state: DebugOptionState,
) {
    let physics_initial_state = [
        state.render_physics_entity.0,
        state.render_physics_tile.0,
    ].as_slice().into();
    let spawn_physics_category: Box<dyn FnOnce(&mut RelatedSpawnerCommands<ChildOf>)> = Box::new(|parent_inner| spawn_debug_category(
        parent_inner,
        "Physics",
        physics_initial_state,
        true,
        |parent| {
            spawn_physics_render_entities_checkbox(parent, state.render_physics_entity.0);
            spawn_physics_render_tiles_checkbox(parent, state.render_physics_tile.0);
        },
    ));


    let nav_initial_state = [
        state.render_nav_map_nodes.0,
        state.render_nav_map_edges.0,
        state.render_npc_paths.0,
    ].as_slice().into();
    let spawn_nav_category: Box<dyn FnOnce(&mut RelatedSpawnerCommands<ChildOf>)> = Box::new(|parent_inner| spawn_debug_category(
        parent_inner,
        "Navigation",
        nav_initial_state,
        true,
        |parent| {
            spawn_render_nav_map_nodes_checkbox(parent, state.render_nav_map_nodes.0);
            spawn_render_nav_map_edges_checkbox(parent, state.render_nav_map_edges.0);
            spawn_render_npc_paths_checkbox(parent, state.render_npc_paths.0);
        },
    ));

    spawn_debug_category(
        parent,
        "Level Renderers",
        physics_initial_state.merge(nav_initial_state),
        true,
        |parent_inner| {
            spawn_physics_category(parent_inner);
            spawn_nav_category(parent_inner);
        },
    );

    spawn_debug_category(
        parent,
        "UI",
        [
            state.render_ui_debug.enabled,
        ].as_slice().into(),
        true,
        |parent| {
            spawn_checkbox_row(
                parent,
                "UI Debug Overlay",
                state.render_ui_debug.enabled,
                DebugUi,
            );
        },
    );

    spawn_debug_category(
        parent,
        "States",
        [
            state.log_screen_transitions.0,
        ].as_slice().into(),
        true,
        |parent| {
            spawn_log_screen_state_checkbox(parent, state.log_screen_transitions.0);
        },
    );
}

#[derive(Component, Debug, Clone)]
struct DebugUi;
impl DebugSetting for DebugUi {}

debug_menu_event!(
    DebugUi,
    fn on_ui_debug(
        event: On<DebugMenuEvent>,
        mut ui_debug_options: ResMut<GlobalUiDebugOptions>,
    ) {
        ui_debug_options.toggle();
        info!("UI Debug toggled: {}", ui_debug_options.enabled);
    }
);