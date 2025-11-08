use crate::AppSystems;
use crate::asset_tracking::LoadResource;
use crate::game::character::health::Health;
use crate::game::character::player::Player;
use crate::game::character::stamina::Stamina;
use crate::screens::Screen;
use crate::theme::prelude::*;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<StatBarAssets>();

    app.add_systems(OnEnter(Screen::Gameplay), spawn_hud);
    app.add_systems(
        Update,
        (update_health_bar, update_stamina_bar)
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Respond),
    );
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
            parent.spawn(stat_bars(&mut texture_atlas_layouts));
        });
}

fn stat_bars(texture_atlas_layouts: &mut Assets<TextureAtlasLayout>) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::new(4, 8), 8, 8, None, None);
    let layout = texture_atlas_layouts.add(layout);

    (
        StatBarLayout(layout),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,

            left: percent(2),
            width: percent(78),

            top: percent(5),
            height: percent(10),

            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((HealthBar, Node::default()));
            parent.spawn((StaminaBar, Node::default()));
        })),
    )
}

#[derive(Component, Debug, Clone)]
struct StatBarLayout(Handle<TextureAtlasLayout>);

#[derive(Component, Debug, Clone, Copy)]
struct HealthBar;
#[derive(Component, Debug, Clone, Copy)]
struct HealthBarSegment;

#[derive(Component, Debug, Clone, Copy)]
struct StaminaBar;
#[derive(Component, Debug, Clone, Copy)]
struct StaminaBarSegment;

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

fn update_health_bar(
    player_query: Query<&Health, With<Player>>,
    stat_bar_layout_query: Query<&StatBarLayout>,
    health_bar_query: Query<Entity, With<HealthBar>>,
    segment_query: Query<Entity, With<HealthBarSegment>>,
    stat_bar_assets: Res<StatBarAssets>,
    mut commands: Commands,
) {
    let Ok(health) = player_query.single() else {
        return;
    };

    let Ok(bar_layout) = stat_bar_layout_query.single() else {
        return;
    };

    let Ok(bar_entity) = health_bar_query.single() else {
        return;
    };

    segment_query
        .iter()
        .for_each(|segment| commands.entity(segment).despawn());

    let texture_atlas_layout = bar_layout.0.clone();

    spawn_stat_bar(
        health.max,
        health.current,
        HEALTH_BAR_PIXEL_VALUE,
        8,
        HealthBarSegment,
        bar_entity,
        &stat_bar_assets,
        &texture_atlas_layout,
        commands,
    )
}

const STAMINA_BAR_PIXEL_VALUE: usize = 10;

fn update_stamina_bar(
    player_query: Query<&Stamina, With<Player>>,
    stat_bar_layout_query: Query<&StatBarLayout>,
    health_bar_query: Query<Entity, With<StaminaBar>>,
    segment_query: Query<Entity, With<StaminaBarSegment>>,
    stat_bar_assets: Res<StatBarAssets>,
    mut commands: Commands,
) {
    let Ok(stamina) = player_query.single() else {
        return;
    };

    let Ok(bar_layout) = stat_bar_layout_query.single() else {
        return;
    };

    let Ok(bar_entity) = health_bar_query.single() else {
        return;
    };

    segment_query
        .iter()
        .for_each(|segment| commands.entity(segment).despawn());

    let texture_atlas_layout = bar_layout.0.clone();

    spawn_stat_bar(
        stamina.max,
        stamina.current as usize,
        HEALTH_BAR_PIXEL_VALUE,
        16,
        StaminaBarSegment,
        bar_entity,
        &stat_bar_assets,
        &texture_atlas_layout,
        commands,
    )
}

fn spawn_stat_bar(
    max: usize,
    current: usize,
    pixel_value: usize,
    bar_sprite_index: usize,
    bar_component: impl Bundle + Clone,
    parent: Entity,
    stat_bar_assets: &StatBarAssets,
    texture_atlas_layout: &Handle<TextureAtlasLayout>,
    mut commands: Commands,
) {
    let segment_value = pixel_value * 3;

    commands.entity(parent).with_children(|parent| {
        let node = Node {
            width: px(4 * 5),
            height: px(8 * 5),
            ..default()
        };

        // Spawn left end cap
        parent.spawn((
            bar_component.clone(),
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
        let num_primary = max / segment_value;
        let num_filled = current / segment_value;

        for _ in 0..num_filled {
            parent.spawn((
                bar_component.clone(),
                ImageNode {
                    image: stat_bar_assets.stat_bars.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: bar_sprite_index,
                    }),
                    ..default()
                },
                node.clone(),
            ));
        }

        if num_filled < num_primary {
            let partial_segment_fill = (current % segment_value) / pixel_value;

            let index = match partial_segment_fill {
                0 => 1,
                1 => bar_sprite_index + 2,
                2 => bar_sprite_index + 1,
                _ => unreachable!(),
            };

            parent.spawn((
                bar_component.clone(),
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
                bar_component.clone(),
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
        let total_bar_remainder = (max % segment_value) / pixel_value;
        let current_bar_remainder = if current > (max / segment_value) * segment_value {
            (current % segment_value) / pixel_value
        } else {
            0
        };

        let indices: &[usize] = match (total_bar_remainder, current_bar_remainder) {
            (1, 1) => &[bar_sprite_index + 7],
            (2, 1) => &[bar_sprite_index + 5, bar_sprite_index + 6],
            (2, 2) => &[bar_sprite_index + 3, bar_sprite_index + 4],
            (1, 0) => &[5],
            (2, 0) => &[3, 4],
            (0, _) => &[2],
            _ => unreachable!(),
        };

        for index in indices {
            parent.spawn((
                bar_component.clone(),
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
