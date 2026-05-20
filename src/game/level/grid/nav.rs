use crate::datagen_api::components::Collider;
use crate::datagen_api::tile::Tile;
use crate::game::level::grid::coords::TileCoords;
use crate::game::level::grid::TileMap;
use bevy::prelude::*;
use getset::{CopyGetters, Getters};
use std::collections::BTreeMap;
use rand::Rng;

pub(in crate::game::level::grid) fn plugin(_app: &mut App) {
}

// Type aliasing done here to allow for conversion without requiring explicit deconstruction,
// nor requiring the caller to care about internal query type information
pub type TileNavQueryParam<'s> = (Entity, &'s Collider);
pub type TileNavQuery<'w, 's> = Query<'w, 's, TileNavQueryParam<'static>, With<Tile>>;

#[derive(Debug, Clone, Copy)]
struct TileNavInfo<'a> {
    // Entity included in struct to preserve the entity reference while still
    // allowing conversion from a tuple using From<TileNavQueryParam>
    entity: Entity,
    collider: &'a Collider
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

#[derive(Component, Default)]
pub struct TileNavMap {
    /// Cached version of the tile map, for change detection if necessary
    _tile_map_cache: TileMap,
    /// Nodes used for pathfinding
    nodes: BTreeMap<TileCoords, NavNode>,
    /// Directed graph of node connections
    edges: BTreeMap<NavEdgeKey, NavEdge>,
    /// Map from a node to outgoing edges
    edges_from: BTreeMap<TileCoords, Vec<NavEdgeKey>>,
}
impl TileNavMap {
    pub fn from_map(
        tile_map: TileMap,
        tile_nav_query: TileNavQuery,
    ) -> Self {
        // Cache tile info
        let mut _self = Self {
            // TileMap is an Alias for Arc, so cloning is trivial
            _tile_map_cache: tile_map.clone(),
            ..Default::default()
        };

        let tile_info = tile_nav_query.into_iter()
            .map(TileNavInfo::from)
            .map(|info| (info.entity, info))
            .collect::<BTreeMap<Entity, TileNavInfo>>();

        let tile_map = tile_map.read().unwrap();

        let tile_info_cache = tile_map.iter()
            .map(|(coords, tile_entity)| (coords.clone(), tile_info[tile_entity]))
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
                    if clearance < 1.0 { clearance = 0.0; }
                    
                    break;
                }
            }
            
            if clearance > 0.0 {
                let node = NavNode {
                    kind: NavNodeKind::Ground,
                    clearance,
                    bounds: tile_info.collider.bounds(),
                };
                _self.add_node(coords.clone(), node);
            }
        }
        
        // Cloned here since we will need mutable access to _self
        let nodes = _self.nodes.clone();
        let directions = [
            IVec3::new(1, 0, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(-1, 0, 0),
            IVec3::new(0, 0, -1),
        ];
        
        // Populate ground tile edges
        for (coords, node) in nodes {
            if node.kind != NavNodeKind::Ground {
                continue;
            }
            
            for dir in directions {
                let new_coords = TileCoords::from(*coords + dir);
                if let Some(new_node) = _self.nodes.get(&new_coords)
                    && new_node.kind == NavNodeKind::Ground
                    && new_node.bounds.1.y - node.bounds.1.y <= MAX_STEP_UP
                    && node.bounds.1.y - new_node.bounds.1.y <= MAX_STEP_DOWN
                {
                    _self.add_edge(
                        NavEdgeKey {
                            start: coords.clone(),
                            end: new_coords.clone(),
                        },
                        NavEdge::new(
                            NavEdgeKind::Walk,
                            node.clearance.min(new_node.clearance)
                        ),
                    );
                }
            }
        }
        
        _self
    }

    fn add_node(&mut self, coords: TileCoords, node: NavNode) {
        self.nodes.insert(coords, node);
    }

    fn add_edge(&mut self, key: NavEdgeKey, edge: NavEdge) {
        self.edges.insert(key.clone(), edge);
        self.edges_from.entry(key.start.clone()).or_default().push(key);
    }

    pub fn has_node(&self, coords: TileCoords) -> bool {
        self.nodes.contains_key(&coords)
    }

    pub fn get_edges_from_tile(&self, coords: TileCoords) -> Option<Vec<(&NavEdgeKey, &NavEdge)>> {
        let edges =
            self.edges_from
                .get(&coords)?
                .iter()
                .map(|key| (key, &self.edges[key]))
                .collect();
        Some(edges)
    }

    pub fn get_edge(&self, start: TileCoords, end: TileCoords) -> Option<&NavEdge> {
        let key = NavEdgeKey { start, end };
        self.edges.get(&key)
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
        Self { start: value.0, end: value.1 }
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
        pub(crate) fn debug_node_positions(&self) -> impl Iterator<Item = Vec3> + '_ {
            self.nodes
                .iter()
                .map(|(coords, node)| nav_node_debug_position(coords, node))
        }

        pub(crate) fn debug_edge_segments(&self) -> impl Iterator<Item = (Vec3, Vec3)> + '_ {
            self.edges
                .keys()
                .filter_map(|key| {
                    let start = self.nodes.get(&key.start)?;
                    let end = self.nodes.get(&key.end)?;

                    Some((
                        nav_node_debug_position(&key.start, start),
                        nav_node_debug_position(&key.end, end),
                    ))
                })
        }
    }

    fn nav_node_debug_position(coords: &TileCoords, node: &NavNode) -> Vec3 {
        let (min, max) = node.bounds;
        let local_center = Vec3::new(
            (min.x + max.x) / 2.0,
            max.y,
            (min.z + max.z) / 2.0,
        );

        coords.0.as_vec3() + local_center
    }
}