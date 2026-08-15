use crate::level::grid::tile::Tile;
use crate::level::grid::{Grid, TileMap};
use crate::level::map::Map;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::{TileCoords, WorldCoords};
use getset::{CopyGetters, Getters};
use physics::Collider;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub(super) fn plugin(_app: &mut App) {}

// Type aliasing done here to allow for conversion without requiring explicit deconstruction,
// nor requiring the caller to care about internal query type information
pub type TileNavQueryParam<'s> = (Entity, &'s Collider);
pub type TileNavQuery<'w, 's> = Query<'w, 's, TileNavQueryParam<'static>, With<Tile>>;

#[derive(SystemParam)]
pub struct NavContext<'w, 's> {
    pub map: Query<'w, 's, (Entity, &'static Children), With<Map>>,
    pub nav_query: TileNavQuery<'w, 's>,
    pub grid: Query<'w, 's, &'static Grid>,
}

#[derive(Debug, Clone, Copy)]
struct TileNavInfo<'a> {
    // Entity included in struct to preserve the entity reference while still
    // allowing conversion from a tuple using From<TileNavQueryParam>
    entity: Entity,
    collider: &'a Collider,
}
impl<'s> From<TileNavQueryParam<'s>> for TileNavInfo<'s> {
    fn from((entity, collider): TileNavQueryParam<'s>) -> Self {
        Self { entity, collider }
    }
}

/// The maximum distance above a node to check for height clearance
const MAX_CLEARANCE: u32 = 5;
const MAX_STEP_UP: f32 = 0.2;
const MAX_STEP_DOWN: f32 = 0.5;

#[derive(Component, Default, Clone)]
pub struct TileNavMap {
    /// Nodes used for pathfinding
    nodes: Arc<RwLock<BTreeMap<TileCoords, NavNode>>>,
    /// Directed graph of node connections
    edges: Arc<RwLock<BTreeMap<NavEdgeKey, NavEdge>>>,
    /// Map from a node to outgoing edges
    edges_from: Arc<RwLock<BTreeMap<TileCoords, Vec<NavEdgeKey>>>>,
}
impl TileNavMap {
    pub fn from_map(tile_map: TileMap, tile_nav_query: TileNavQuery) -> Self {
        // Cache tile info
        let mut slf = Self::default();

        let tile_info = tile_nav_query
            .into_iter()
            .map(TileNavInfo::from)
            .map(|info| (info.entity, info))
            .collect::<BTreeMap<Entity, TileNavInfo>>();

        let tile_map = tile_map.read().unwrap();

        let tile_info_cache = tile_map
            .iter()
            .map(|(coords, tile_entity)| (*coords, tile_info[tile_entity]))
            .collect::<BTreeMap<TileCoords, TileNavInfo>>();

        // Process nodes
        for (coords, tile_info) in tile_info_cache.iter() {
            let mut clearance = MAX_CLEARANCE as f32;
            let (_, current_max) = tile_info.collider.bounds();

            for height in 1..=MAX_CLEARANCE {
                let check_coord = **coords + IVec3::new(0, height as i32, 0);

                if let Some(next_tile_info) = tile_info_cache.get(&check_coord.into()) {
                    let (next_min, _) = next_tile_info.collider.bounds();

                    clearance = next_min.y - current_max.y;
                    if clearance < 1.0 {
                        clearance = 0.0;
                    }

                    break;
                }
            }

            if clearance > 0.0 {
                let node = NavNode {
                    kind: NavNodeKind::Ground,
                    clearance,
                    bounds: tile_info.collider.bounds(),
                };
                slf.add_node(*coords, node);
            }
        }

        // Cloned here since we will need mutable access to _self
        let nodes = slf.nodes.clone();
        let directions = [
            IVec3::new(1, 0, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(-1, 0, 0),
            IVec3::new(0, 0, -1),
        ];

        // Populate ground tile edges
        for (coords, node) in nodes.read().unwrap().iter() {
            if node.kind != NavNodeKind::Ground {
                continue;
            }

            for dir in directions {
                let new_coords = TileCoords::from(**coords + dir);
                let new_node = nodes.read().unwrap().get(&new_coords).cloned();

                if let Some(new_node) = new_node
                    && new_node.kind == NavNodeKind::Ground
                    && new_node.bounds.1.y - node.bounds.1.y <= MAX_STEP_UP
                    && node.bounds.1.y - new_node.bounds.1.y <= MAX_STEP_DOWN
                {
                    slf.add_edge(
                        NavEdgeKey {
                            start: *coords,
                            end: new_coords,
                        },
                        NavEdge::new(NavEdgeKind::Walk, node.clearance.min(new_node.clearance)),
                    );
                }
            }
        }

        slf
    }

    fn add_node(&mut self, coords: TileCoords, node: NavNode) {
        self.nodes.write().unwrap().insert(coords, node);
    }

    fn add_edge(&mut self, key: NavEdgeKey, edge: NavEdge) {
        self.edges.write().unwrap().insert(key.clone(), edge);
        self.edges_from
            .write()
            .unwrap()
            .entry(key.start)
            .or_default()
            .push(key);
    }

    pub fn has_node(&self, coords: &TileCoords) -> bool {
        self.nodes.read().unwrap().contains_key(coords)
    }

    pub fn get_edges_from_tile(&self, coords: &TileCoords) -> Option<Vec<(NavEdgeKey, NavEdge)>> {
        let edges = self
            .edges_from
            .read()
            .unwrap()
            .get(coords)?
            .iter()
            .map(|key| (key.clone(), self.edges.read().unwrap()[key].clone()))
            .collect();
        Some(edges)
    }

    pub fn _get_edge(&self, start: &TileCoords, end: &TileCoords) -> Option<NavEdge> {
        let key = NavEdgeKey {
            start: *start,
            end: *end,
        };
        self.edges.read().unwrap().get(&key).cloned()
    }

    /// Check for line-of-sight between two tiles with the given clearance.
    ///
    /// Note that this only works on the same y level.
    pub fn has_line_of_sight(
        &self,
        start: &TileCoords,
        end: &TileCoords,
        clearance_half_width: f32,
        clearance_height: f32,
    ) -> bool {
        if !self.has_node(start) || !self.has_node(end) {
            return false;
        }

        // Line of sight checks only work on the same y level
        if start.y != end.y {
            return false;
        }

        let dir = **end - **start;
        let dir = dir.as_vec3().normalize();

        let offset = Vec3::new(-dir.z, 0.0, dir.x) * clearance_half_width;

        let left_start = start.as_vec3() + offset;
        let left_end = end.as_vec3() + offset;

        let right_start = start.as_vec3() - offset;
        let right_end = end.as_vec3() - offset;

        let left_coords = Self::get_intersecting_coords(&left_start.into(), &left_end.into());
        let right_coords = Self::get_intersecting_coords(&right_start.into(), &right_end.into());

        self.validate_coords_for_path(&left_coords, clearance_height)
            && self.validate_coords_for_path(&right_coords, clearance_height)
    }

    fn validate_coords_for_path(&self, coords: &[TileCoords], clearance_height: f32) -> bool {
        coords.iter().all(|coord| {
            if !self.has_node(coord) {
                return false;
            }
            let nodes = self.nodes.read().unwrap();
            let node = nodes.get(coord).unwrap();
            node.kind == NavNodeKind::Ground && node.clearance >= clearance_height
        })
    }

    /// Finds all tiles intersected by the ray between two points
    fn get_intersecting_coords(start: &WorldCoords, end: &WorldCoords) -> Vec<TileCoords> {
        let mut intersections = Vec::new();

        // Store the start and end positions as points along our path
        intersections.push(Vec2::new(start.x, start.z));
        intersections.push(Vec2::new(end.x, end.z));

        let dx = end.x - start.x;
        let dz = end.z - start.z;

        // --- 1. Find all X-axis grid boundary crossings ---
        if dx.abs() > 0.000001 {
            // Grid lines sit on the half-integers (e.g., -0.5, 0.5, 1.5)
            // Find the range of grid boundaries bounded by start and end
            let min_x = start.x.min(end.x);
            let max_x = start.x.max(end.x);

            let first_boundary = (min_x - 0.5).ceil() + 0.5;
            let last_boundary = (max_x - 0.5).floor() + 0.5;

            let mut current_boundary = first_boundary;
            while current_boundary <= last_boundary {
                // Linear interpolation: find Z at this specific X boundary
                let t = (current_boundary - start.x) / dx;
                let z_intersect = start.z + t * dz;
                intersections.push(Vec2::new(current_boundary, z_intersect));
                current_boundary += 1.0;
            }
        }

        // --- 2. Find all Z-axis grid boundary crossings ---
        if dz.abs() > 0.000001 {
            let min_z = start.z.min(end.z);
            let max_z = start.z.max(end.z);

            let first_boundary = (min_z - 0.5).ceil() + 0.5;
            let last_boundary = (max_z - 0.5).floor() + 0.5;

            let mut current_boundary = first_boundary;
            while current_boundary <= last_boundary {
                // Linear interpolation: find X at this specific Z boundary
                let t = (current_boundary - start.z) / dz;
                let x_intersect = start.x + t * dx;
                intersections.push(Vec2::new(x_intersect, current_boundary));
                current_boundary += 1.0;
            }
        }

        // --- 3. Sort intersections along the travel direction ---
        // We sort based on distance from the start point
        let start_point = Vec2::new(start.x, start.z);
        intersections.sort_by(|a, b| {
            let dist_a = a.distance_squared(start_point);
            let dist_b = b.distance_squared(start_point);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // --- 4. Convert segments to unique TileCoords ---
        let mut coords = Vec::new();
        let y_level = start.y as i32;

        // Every adjacent pair of sorted intersection points represents a line segment
        // passing entirely inside a single grid tile.
        for window in intersections.windows(2) {
            let p1 = window[0];
            let p2 = window[1];

            // Find the absolute midpoint of the segment within the tile
            let midpoint = (p1 + p2) * 0.5;

            // Since tile centers are whole integers, rounding the midpoint
            // to the nearest integer cleanly gives us the exact tile coordinate.
            let tile_x = midpoint.x.round() as i32;
            let tile_z = midpoint.y.round() as i32;

            let coord = TileCoords::from(IVec3::new(tile_x, y_level, tile_z));

            // Deduplicate coordinates (points resting directly on boundaries
            // can occasionally generate adjacent duplicates)
            if coords.last() != Some(&coord) {
                coords.push(coord);
            }
        }

        coords
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct NavNode {
    kind: NavNodeKind,
    clearance: f32,
    bounds: (Vec3, Vec3),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NavNodeKind {
    Ground,
    //Slope(TileFacing),
}

/// Directional edge between two tiles
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Getters)]
pub struct NavEdgeKey {
    #[getset(get = "pub")]
    start: TileCoords,
    #[getset(get = "pub")]
    end: TileCoords,
}
impl From<(TileCoords, TileCoords)> for NavEdgeKey {
    fn from(value: (TileCoords, TileCoords)) -> Self {
        Self {
            start: value.0,
            end: value.1,
        }
    }
}
impl From<NavEdgeKey> for (TileCoords, TileCoords) {
    fn from(value: NavEdgeKey) -> Self {
        (value.start, value.end)
    }
}

#[derive(Debug, Clone, PartialEq, CopyGetters)]
pub struct NavEdge {
    #[getset(get_copy = "pub")]
    kind: NavEdgeKind,
    #[getset(get_copy = "pub")]
    clearance: f32,
    #[getset(get_copy = "pub")]
    cost: u32,
}
impl NavEdge {
    fn new(kind: NavEdgeKind, clearance: f32) -> Self {
        Self {
            kind,
            clearance,
            cost: kind.base_cost(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavEdgeKind {
    Walk,
    _SlopeUp,
    _SlopeDown,
    _Jump,
    _Drop,
}
impl NavEdgeKind {
    fn base_cost(&self) -> u32 {
        match self {
            NavEdgeKind::Walk => 10,
            NavEdgeKind::_SlopeUp => 15,
            NavEdgeKind::_SlopeDown => 10,
            NavEdgeKind::_Jump => 30,
            NavEdgeKind::_Drop => 8,
        }
    }
}

#[cfg(feature = "dev")]
mod debug_helpers {
    use super::*;

    impl TileNavMap {
        pub fn debug_node_positions(&self) -> Vec<Vec3> {
            self.nodes
                .read()
                .unwrap()
                .iter()
                .map(|(coords, node)| nav_node_debug_position(coords, node))
                .collect()
        }

        pub fn debug_edge_segments(&self) -> Vec<(Vec3, Vec3)> {
            self.edges
                .read()
                .unwrap()
                .keys()
                .filter_map(|key| {
                    let nodes = self.nodes.read().unwrap();

                    let start = nodes.get(&key.start)?;
                    let end = nodes.get(&key.end)?;

                    Some((
                        nav_node_debug_position(&key.start, start),
                        nav_node_debug_position(&key.end, end),
                    ))
                })
                .collect()
        }
    }

    fn nav_node_debug_position(coords: &TileCoords, node: &NavNode) -> Vec3 {
        let (min, max) = node.bounds;
        let local_offset = Vec3::Y * ((max.y - min.y) - 0.5);
        coords.0.as_vec3() + local_offset
    }
}
