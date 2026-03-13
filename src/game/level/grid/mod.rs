use crate::game::character::player::Player;
use crate::game::level::grid::coords::{SCREEN_Z_SCALE, TileCoords, TilePosition, WorldPosition};
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

/// Iterates through objects in the current world. Starting from a threshold, any that are above
/// the player are faded out until they fully disappear.
fn hide_objects_above(
    mut query: Query<(&mut Sprite, &WorldPosition, Option<&Children>)>,
    mut child_query: Query<(Entity, &mut Sprite), (Without<TilePosition>, Without<WorldPosition>)>,
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

            set_alpha(child_query.reborrow(), children, alpha);
        })
}

/// Iterates through tiles in the current world. Starting from a threshold, any that are above
/// the player are faded out until they fully disappear.
fn hide_tiles_above(
    mut query: Query<(&mut Sprite, &TilePosition, Option<&Children>)>,
    mut child_query: Query<(Entity, &mut Sprite), (Without<TilePosition>, Without<WorldPosition>)>,
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

            set_alpha(child_query.reborrow(), children, alpha);
        })
}

/// Iterates through all sprites of an entity and its children, setting their alpha value.
fn set_alpha(
    mut child_query: Query<(Entity, &mut Sprite), (Without<TilePosition>, Without<WorldPosition>)>,
    children: Option<&Children>,
    alpha: f32,
) {
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
}

/// Corrects the shadow opacity for objects after their main opacity has been faded for y height
// TODO: This is a temporary fix for the shadows being too dark. This should be fixed in the shadow
//       component itself, or some other more appropriate way
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

pub type TileMap = Arc<RwLock<BTreeMap<TileCoords, Entity>>>;
pub fn tile_map() -> TileMap {
    Arc::new(RwLock::new(BTreeMap::new()))
}

/// Merge two tile maps, offsetting the other grid by the provided offset
///
/// Returns a result indicating success or failure of the merge operation
///
/// ### OverlapsExistingGrid:
/// The provided grid overlaps with the existing grid.
/// Note that this is smart enough to handle gaps, so the extents may overlap,
/// but it does not handle overlapping tiles.
///
/// The error returns a list of overlapping tiles so that the caller may handle the overlap
/// and try again. Note that the overlaps are relative to the original grid. To get coordinates
/// relative to the other grid, subtract the offset from the returned coordinates.
pub fn merge_tile_map(map: &TileMap, other: TileMap, offset: IVec3) -> Result<(), TileMapMergeError> {
    let mut overlaps = Vec::new();

    for (coords, tile_entity) in &*other.read().unwrap() {
        let coords = TileCoords(coords.0 + offset);
        map.write().unwrap()
            .entry(coords.clone())
            .and_modify(|_| {
                overlaps.push(MergeTileCoords {
                    coords: coords.clone(),
                    offset,
                });
            })
            .or_insert(*tile_entity);
    }

    if !overlaps.is_empty() {
        return Err(TileMapMergeError::OverlapsExistingGrid(overlaps));
    }

    Ok(())
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TileMapMergeError {
    #[error("New grid overlaps with existing grid")]
    OverlapsExistingGrid(Vec<MergeTileCoords>)
}

#[derive(Debug, Clone)]
pub struct MergeTileCoords {
    pub coords: TileCoords,
    pub offset: IVec3,
}

impl MergeTileCoords {
    pub fn original_grid_coords(&self) -> IVec3 {
        self.coords.0
    }

    pub fn other_grid_coords(&self) -> IVec3 {
        self.coords.0 - self.offset
    }
}

#[derive(Component, Clone)]
pub struct Grid(TileMap);
impl Grid {
    pub fn new(tile_map: TileMap) -> Self {
        Self(tile_map)
    }

    pub fn tile_map(&self) -> &TileMap {
        &self.0
    }

    pub fn tile_map_mut(&mut self) -> &mut TileMap {
        &mut self.0
    }

    /// The absolute size of the grid, ignoring the actual coordinate values
    pub fn size(&self) -> UVec3 {
        if self.0.read().unwrap().is_empty() {
            return UVec3::ZERO;
        }

        let size = self.extent().1 - self.extent().0 + IVec3::ONE;
        UVec3::new(size.x as u32, size.y as u32, size.z as u32)
    }

    /// The minimum and maximum coordinates of the grid
    pub fn extent(&self) -> (IVec3, IVec3) {
        if self.0.read().unwrap().is_empty() {
            return (IVec3::ZERO, IVec3::ZERO);
        }

        let mut min = IVec3::new(i32::MAX, i32::MAX, i32::MAX);
        let mut max = IVec3::new(i32::MIN, i32::MIN, i32::MIN);

        for coords in self.0.read().unwrap().keys() {
            min = min.min(coords.0);
            max = max.max(coords.0);
        }

        (min, max)
    }
}

pub fn grid_bundle(grid: Grid, scale: f32) -> impl Bundle {
    (
        grid,
        Transform::from_scale(Vec2::splat(scale).extend(SCREEN_Z_SCALE)),
        InheritedVisibility::default(),
    )
}