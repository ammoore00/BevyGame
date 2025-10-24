use std::time::Duration;
use bevy::camera::NormalizedRenderTarget;
use bevy::input_focus::directional_navigation::{DirectionalNavigation, DirectionalNavigationPlugin};
use bevy::input_focus::{InputDispatchPlugin, InputFocus, InputFocusVisible};
use bevy::math::CompassOctant;
use bevy::picking::backend::HitData;
use bevy::picking::pointer::{Location, PointerId};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app
        // Input focus is not enabled by default, so we need to add the corresponding plugins
        .add_plugins((
            InputDispatchPlugin,
            DirectionalNavigationPlugin,
        ))
        // This resource is canonically used to track whether or not to render a focus indicator
        // It starts as false, but we set it to true here as we would like to see the focus indicator
        .insert_resource(InputFocusVisible(true))
        // We've made a simple resource to keep track of the actions that are currently being pressed for this example
        .init_resource::<ActionState>()
        .add_systems(PreUpdate, (process_inputs, navigate).chain())
        .add_systems(
            Update,
            (
                // Pressing the "Interact" button while we have a focused element should simulate a click
                interact_with_focused_button,
            ),
        );
}

// The indirection between inputs and actions allows us to easily remap inputs
// and handle multiple input sources (keyboard, gamepad, etc.) in our game
#[derive(Debug, PartialEq, Eq, Hash)]
enum DirectionalNavigationAction {
    Up,
    Down,
    Left,
    Right,
    Select,
}

impl DirectionalNavigationAction {
    fn variants() -> Vec<Self> {
        vec![
            DirectionalNavigationAction::Up,
            DirectionalNavigationAction::Down,
            DirectionalNavigationAction::Left,
            DirectionalNavigationAction::Right,
            DirectionalNavigationAction::Select,
        ]
    }

    fn keycode(&self) -> KeyCode {
        match self {
            DirectionalNavigationAction::Up => KeyCode::ArrowUp,
            DirectionalNavigationAction::Down => KeyCode::ArrowDown,
            DirectionalNavigationAction::Left => KeyCode::ArrowLeft,
            DirectionalNavigationAction::Right => KeyCode::ArrowRight,
            DirectionalNavigationAction::Select => KeyCode::Enter,
        }
    }

    fn gamepad_button(&self) -> GamepadButton {
        match self {
            DirectionalNavigationAction::Up => GamepadButton::DPadUp,
            DirectionalNavigationAction::Down => GamepadButton::DPadDown,
            DirectionalNavigationAction::Left => GamepadButton::DPadLeft,
            DirectionalNavigationAction::Right => GamepadButton::DPadRight,
            // This is the "A" button on an Xbox controller,
            // and is conventionally used as the "Select" / "Interact" button in many games
            DirectionalNavigationAction::Select => GamepadButton::South,
        }
    }
}

// This keeps track of the inputs that are currently being pressed
#[derive(Default, Resource)]
struct ActionState {
    pressed_actions: HashSet<DirectionalNavigationAction>,
}

fn process_inputs(
    mut action_state: ResMut<ActionState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepad_input: Query<&Gamepad>,
    mut commands: Commands,
) {
    // Reset the set of pressed actions each frame
    // to ensure that we only process each action once
    action_state.pressed_actions.clear();

    for action in DirectionalNavigationAction::variants() {
        // Use just_pressed to ensure that we only process each action once
        // for each time it is pressed
        if keyboard_input.just_pressed(action.keycode()) {
            commands.insert_resource(InputFocusVisible(true));
            action_state.pressed_actions.insert(action);
        }
    }

    // We're treating this like a single-player game:
    // if multiple gamepads are connected, we don't care which one is being used
    for gamepad in gamepad_input.iter() {
        for action in DirectionalNavigationAction::variants() {
            // Unlike keyboard input, gamepads are bound to a specific controller
            if gamepad.just_pressed(action.gamepad_button()) {
                action_state.pressed_actions.insert(action);
            }
        }
    }
}

fn navigate(
    action_state: Res<ActionState>,
    mut directional_navigation: DirectionalNavigation,
) {
    // If the user is pressing both left and right, or up and down,
    // we should not move in either direction.
    let net_east_west = action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Right) as i8
        - action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Left) as i8;

    let net_north_south = action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Up) as i8
        - action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Down) as i8;

    // Compute the direction that the user is trying to navigate in
    let maybe_direction = match (net_east_west, net_north_south) {
        (0, 0) => None,
        (0, 1) => Some(CompassOctant::North),
        (1, 1) => Some(CompassOctant::NorthEast),
        (1, 0) => Some(CompassOctant::East),
        (1, -1) => Some(CompassOctant::SouthEast),
        (0, -1) => Some(CompassOctant::South),
        (-1, -1) => Some(CompassOctant::SouthWest),
        (-1, 0) => Some(CompassOctant::West),
        (-1, 1) => Some(CompassOctant::NorthWest),
        _ => None,
    };

    if let Some(direction) = maybe_direction {
        match directional_navigation.navigate(direction) {
            // In a real game, you would likely want to play a sound or show a visual effect
            // on both successful and unsuccessful navigation attempts
            Ok(entity) => {
                println!("Navigated {direction:?} successfully. {entity} is now focused.");
            }
            Err(e) => println!("Navigation failed: {e}"),
        }
    }
}

// By sending a Pointer<Click> trigger rather than directly handling button-like interactions,
// we can unify our handling of pointer and keyboard/gamepad interactions
fn interact_with_focused_button(
    action_state: Res<ActionState>,
    input_focus: Res<InputFocus>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    if action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Select)
        && let Some(focused_entity) = input_focus.0
        && let Ok(children) = children.get(focused_entity)
    {
        children.iter().for_each(|child| {
            commands.trigger(Pointer::<Click> {
                entity: child,
                // We're pretending that we're a mouse
                pointer_id: PointerId::Mouse,
                // This field isn't used, so we're just setting it to a placeholder value
                pointer_location: Location {
                    target: NormalizedRenderTarget::None {
                        width: 0,
                        height: 0,
                    },
                    position: Vec2::ZERO,
                },
                event: Click {
                    button: PointerButton::Primary,
                    // This field isn't used, so we're just setting it to a placeholder value
                    hit: HitData {
                        camera: Entity::PLACEHOLDER,
                        depth: 0.0,
                        position: None,
                        normal: None,
                    },
                    duration: Duration::from_secs_f32(0.1),
                },
            });
        });
    }
}