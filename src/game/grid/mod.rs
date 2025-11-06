use crate::game::character::player::Player;
use crate::game::grid::coords::{SCREEN_Z_SCALE, TileCoords, TilePosition, WorldPosition};
pub(crate) use crate::game::grid::tile::TileAssets;
use crate::game::object::Shadow;
use bevy::prelude::*;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub mod coords;
pub mod tile;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((coords::plugin, tile::plugin));

    app.add_systems(
        PreUpdate,
        (
            (hide_tiles_above, hide_objects_above),
            correct_shadow_opacity,
        )
            .chain(),
    );
}

fn hide_objects_above(
    mut query: Query<(&mut Sprite, &WorldPosition, Option<&Children>)>,
    mut child_query: Query<(Entity, &mut Sprite), Without<WorldPosition>>,
    player_query: Query<&WorldPosition, With<Player>>,
) {
    let player_height = if let Ok(player_pos) = player_query.single() {
        player_pos.0.y
    } else {
        return;
    };

    let player_height = ((player_height + 0.05) * 12.0).round() / 12.0;

    query
        .iter_mut()
        .for_each(|(ref mut sprite, world_position, children)| {
            let world_height = (world_position.0.y - 1.0).round();

            let mut alpha = (1.0 - ((world_height - player_height) - 1.0) * 0.25).clamp(0.0, 1.0);

            if alpha < 0.99 {
                alpha *= 0.5;
            }

            let prev_color = sprite.color.to_srgba();
            sprite.color = Color::srgba(prev_color.red, prev_color.green, prev_color.blue, alpha);

            if let Some(children) = children {
                child_query
                    .iter_mut()
                    .for_each(|(child_entity, ref mut child_sprite)| {
                        if children.contains(&child_entity) {
                            let child_prev_color = child_sprite.color.to_srgba();
                            child_sprite.color = Color::srgba(
                                child_prev_color.red,
                                child_prev_color.green,
                                child_prev_color.blue,
                                alpha,
                            );
                        }
                    })
            }
        })
}

fn hide_tiles_above(
    mut query: Query<(&mut Sprite, &TilePosition, Option<&Children>)>,
    mut child_query: Query<(Entity, &mut Sprite), Without<TilePosition>>,
    player_query: Query<&WorldPosition, With<Player>>,
) {
    let player_height = if let Ok(player_pos) = player_query.single() {
        player_pos.0.y
    } else {
        return;
    };

    let player_height = ((player_height + 0.05) * 12.0).round() / 12.0;

    query
        .iter_mut()
        .for_each(|(ref mut sprite, tile_position, children)| {
            let tile_height = tile_position.0.y;

            let mut alpha =
                (1.0 - ((tile_height as f32 - player_height) - 1.0) * 0.25).clamp(0.0, 1.0);

            if alpha < 0.99 {
                alpha *= 0.5;
            }

            let prev_color = sprite.color.to_srgba();
            sprite.color = Color::srgba(prev_color.red, prev_color.green, prev_color.blue, alpha);

            if let Some(children) = children {
                child_query
                    .iter_mut()
                    .for_each(|(child_entity, ref mut child_sprite)| {
                        if children.contains(&child_entity) {
                            let child_prev_color = child_sprite.color.to_srgba();
                            child_sprite.color = Color::srgba(
                                child_prev_color.red,
                                child_prev_color.green,
                                child_prev_color.blue,
                                alpha,
                            );
                        }
                    })
            }
        })
}

fn correct_shadow_opacity(mut query: Query<&mut Sprite, With<Shadow>>) {
    query.iter_mut().for_each(|mut sprite| {
        let prev_color = sprite.color.to_srgba();
        sprite.color = Color::srgba(
            prev_color.red,
            prev_color.green,
            prev_color.blue,
            prev_color.alpha * 0.75,
        );
    })
}

#[derive(Component)]
pub struct Grid;

pub fn grid(_tile_map: Arc<RwLock<BTreeMap<TileCoords, Entity>>>, scale: f32) -> impl Bundle {
    (
        Grid,
        Transform::from_scale(Vec2::splat(scale).extend(SCREEN_Z_SCALE)),
        InheritedVisibility::default(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Facing {
    NorthWest = 0,
    West = 1,
    SouthWest = 2,
    South = 3,
    SouthEast = 4,
    East = 5,
    NorthEast = 6,
    North = 7,
}

impl From<usize> for Facing {
    fn from(index: usize) -> Self {
        match index {
            0 => Self::NorthWest,
            1 => Self::West,
            2 => Self::SouthWest,
            3 => Self::South,
            4 => Self::SouthEast,
            5 => Self::East,
            6 => Self::NorthEast,
            7 => Self::North,
            _ => unreachable!(),
        }
    }
}

impl From<Vec2> for Facing {
    fn from(vec: Vec2) -> Self {
        // Calculate angle in radians (-PI to PI)
        // Note: atan2(z, x) where x is "forward" and z is "right"
        let angle = vec.x.atan2(vec.y);

        // Convert to 0-8 range, where each direction occupies 45 degrees (PI/4 radians)
        // Add PI to shift range from [-PI, PI] to [0, 2*PI]
        // Add PI/8 to center the divisions on the cardinal directions
        // Add 3PI/2 to rotate divisions to align with sprite sheets
        // Divide by PI/4 (45 degrees) to get 0-8 range
        let direction_index = ((angle
            + std::f32::consts::PI
            + std::f32::consts::FRAC_PI_8
            + std::f32::consts::FRAC_PI_2 * 3.0)
            / std::f32::consts::FRAC_PI_4)
            .floor() as i32
            % 8;

        Self::from(direction_index as usize)
    }
}