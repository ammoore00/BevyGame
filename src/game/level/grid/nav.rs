use crate::datagen_api::components::Collider;
use crate::datagen_api::tile::{Tile, TileFacing};
use crate::game::level::grid::coords::TileCoords;
use crate::game::level::grid::TileMap;
use bevy::prelude::*;
use getset::CopyGetters;
use std::collections::BTreeMap;

pub(in crate::game::level::grid) fn plugin(_app: &mut App) {
}

/// High-level map between rooms
#[derive(Component, Debug, Clone)]
pub struct LevelNavMap {

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

/// Low-level map for tiles within a room
#[derive(Component, Default)]
pub struct TileNavMap {
    /// Cached version of the tile map, for change detection if necessary
    tile_map_cache: TileMap,
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
        let mut _self = Self {
            // TileMap is an Alias for Arc, so cloning is trivial
            tile_map_cache: tile_map.clone(),
            ..Default::default()
        };
        
        // Cache tile info
        let tile_info = tile_nav_query.into_iter()
            .map(TileNavInfo::from)
            .map(|info| (info.entity, info))
            .collect::<BTreeMap<Entity, TileNavInfo>>();

        let tile_map = tile_map.read().unwrap();

        info!("Tile map: {:?}", tile_map.iter());
        info!("Tile info: {:?}", tile_info.iter());

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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NavEdgeKey {
    start: TileCoords,
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
struct NavEdge {
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
enum NavEdgeKind {
    Walk,
    SlopeUp,
    SlopeDown,
    Jump,
    Drop,
}
impl NavEdgeKind {
    fn base_cost(&self) -> u32 {
        match self {
            NavEdgeKind::Walk => 10,
            NavEdgeKind::SlopeUp => 15,
            NavEdgeKind::SlopeDown => 10,
            NavEdgeKind::Jump => 30,
            NavEdgeKind::Drop => 8,
        }
    }
}