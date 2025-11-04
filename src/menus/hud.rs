use bevy::prelude::*;
use crate::theme::prelude::*;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_hud);
    app.add_systems(Update, update_hud.run_if(in_state(Screen::Gameplay)));
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct HealthText;

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        HudRoot,
        widget::ui_root("HUD"),
        GlobalZIndex(1),
        DespawnOnExit(Screen::Gameplay),
    )).with_children(|parent| {
        // Add your HUD elements here
        parent.spawn((
            HealthText,
            Text::new("Health: 100"),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,

                left: percent(40),
                width: percent(20),

                top: percent(2),
                height: px(30),

                ..default()
            }
        ));
    });
}

fn update_hud(
    // Query your game state here
    mut health_text: Query<&mut Text, With<HealthText>>,
) {
    // Update HUD based on game state
}

fn despawn_hud(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    for entity in &hud {
        commands.entity(entity).despawn();
    }
}