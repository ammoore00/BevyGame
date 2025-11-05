use crate::asset_tracking::LoadResource;
use crate::game::character::health::Health;
use crate::game::character::player::Player;
use crate::screens::Screen;
use crate::theme::prelude::*;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<StatBarAssets>();

    app.add_systems(OnEnter(Screen::Gameplay), spawn_hud);
    app.add_systems(Update, update_hud.run_if(in_state(Screen::Gameplay)));
}

#[derive(Component, Debug, Clone)]
struct HudRoot;

fn spawn_hud(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands
        .spawn((
            HudRoot,
            widget::ui_root("HUD"),
            GlobalZIndex(1),
            DespawnOnExit(Screen::Gameplay),
        ))
        .with_children(|parent| {
            // Add your HUD elements here
            parent.spawn(health(&mut texture_atlas_layouts));
        });
}

fn health(texture_atlas_layouts: &mut Assets<TextureAtlasLayout>) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::new(4, 8), 8, 8, None, None);
    let layout = texture_atlas_layouts.add(layout);

    (
        HealthBarContainer(layout),
        Node {
            position_type: PositionType::Absolute,

            left: percent(2),
            width: percent(78),

            top: percent(5),
            height: px(30),

            ..default()
        },
    )
}

#[derive(Component)]
struct HealthBarContainer(Handle<TextureAtlasLayout>);

#[derive(Component)]
struct HealthBarSegment;

#[derive(Resource, Asset, Clone, Reflect)]
pub struct StatBarAssets {
    #[dependency]
    stat_bars: Handle<Image>,
}

impl FromWorld for StatBarAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            stat_bars: assets.load("images/ui/bars.png"),
        }
    }
}

const HEALTH_BAR_PIXEL_VALUE: usize = 10;
const HEALTH_BAR_SEGMENT_VALUE: usize = HEALTH_BAR_PIXEL_VALUE * 3;

fn update_hud(
    player_query: Query<&Health, With<Player>>,
    health_bar_query: Query<(Entity, &HealthBarContainer)>,
    segment_query: Query<Entity, With<HealthBarSegment>>,
    stat_bar_assets: Res<StatBarAssets>,
    mut commands: Commands,
) {
    let Ok(health) = player_query.single() else {
        return;
    };

    let Ok((health_bar_entity, health_bar_container)) = health_bar_query.single() else {
        return;
    };

    segment_query
        .iter()
        .for_each(|segment| commands.entity(segment).despawn());

    let texture_atlas_layout = health_bar_container.0.clone();

    commands.entity(health_bar_entity).with_children(|parent| {
        let node = Node {
            width: px(4 * 4),
            height: px(8 * 4),
            ..default()
        };

        // Spawn left end cap
        parent.spawn((
            HealthBarSegment,
            ImageNode {
                image: stat_bar_assets.stat_bars.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            node.clone(),
        ));

        // Spawn main health bar
        let num_primary = health.max / HEALTH_BAR_SEGMENT_VALUE;
        let num_filled = health.current / HEALTH_BAR_SEGMENT_VALUE;

        for _ in 0..num_filled {
            parent.spawn((
                HealthBarSegment,
                ImageNode {
                    image: stat_bar_assets.stat_bars.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: 8,
                    }),
                    ..default()
                },
                node.clone(),
            ));
        }

        if num_filled < num_primary {
            let partial_segment_fill =
                (health.current % HEALTH_BAR_SEGMENT_VALUE) / HEALTH_BAR_PIXEL_VALUE;

            let index = match partial_segment_fill {
                0 => 1,
                1 => 10,
                2 => 9,
                _ => unreachable!(),
            };

            parent.spawn((
                HealthBarSegment,
                ImageNode {
                    image: stat_bar_assets.stat_bars.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index,
                    }),
                    ..default()
                },
                node.clone(),
            ));
        }

        // Minus one is due to partial fill handling - even if the middle segment is fully empty
        let num_empty = (num_primary as i32 - num_filled as i32 - 1).max(0) as usize;

        for _ in 0..num_empty {
            parent.spawn((
                HealthBarSegment,
                ImageNode {
                    image: stat_bar_assets.stat_bars.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: 1,
                    }),
                    ..default()
                },
                node.clone(),
            ));
        }

        // Spawn right end cap
        let remainder_health = (health.max % HEALTH_BAR_SEGMENT_VALUE) / HEALTH_BAR_PIXEL_VALUE;
        let remainder_current_health = if health.current > health.max % HEALTH_BAR_SEGMENT_VALUE {
            (health.current % HEALTH_BAR_SEGMENT_VALUE) / HEALTH_BAR_PIXEL_VALUE
        } else {
            0
        };

        let indices: &[usize] = match (remainder_health, remainder_current_health) {
            (1, 1) => &[15],
            (2, 1) => &[13, 14],
            (2, 2) => &[11, 12],
            (1, 0) => &[5],
            (2, 0) => &[3, 4],
            (0, _) => &[2],
            _ => unreachable!(),
        };

        for index in indices {
            parent.spawn((
                HealthBarSegment,
                ImageNode {
                    image: stat_bar_assets.stat_bars.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: *index,
                    }),
                    ..default()
                },
                node.clone(),
            ));
        }
    });
}
