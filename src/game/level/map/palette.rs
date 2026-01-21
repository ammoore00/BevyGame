use bevy::prelude::*;
use crate::game::level::map::connector::ConnectorPool;
use crate::game::level::map::MapPool;
use crate::game::level::map::room::RoomRegistryContext;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<Palette>();
    app.init_resource::<Palettes>();
}

#[derive(Resource, Asset, Debug, Reflect)]
pub struct Palettes {
    pub standard: Handle<Palette>,
}

impl FromWorld for Palettes {
    fn from_world(world: &mut World) -> Self {
        let mut context = world.resource_mut::<RoomRegistryContext>();
        let standard_palette = standard::palette(&mut context);

        let assets = world.resource::<AssetServer>();

        Self {
            standard: assets.add(standard_palette),
        }
    }
}

#[derive(Asset, Debug, Reflect)]
pub struct Palette {
    main_map_pool: MapPool,
    connector_pool: ConnectorPool,
}

impl Palette {
    pub fn connector_pool(&self) -> &ConnectorPool {
        &self.connector_pool
    }
    
    pub fn main_map_pool(&self) -> &MapPool {
        &self.main_map_pool
    }
}

mod standard {
    use crate::game::level::map::room::RoomRegistryContext;
    use super::*;

    pub(super) fn palette(
        context: &mut RoomRegistryContext,
    ) -> Palette {
        Palette {
            main_map_pool: maps::main_map_pool(),
            connector_pool: connectors::connector_pool(context),
        }
    }
    
    mod maps {
        use crate::game::level::map::{MapDefinition, MapType};
        use super::*;
        
        pub(super) fn main_map_pool() -> MapPool {
            let main_map = MapDefinition {
                map_type: MapType::Main,
                map_size: 3,
            };
            MapPool(vec![main_map])
        }
    }

    mod connectors {
        use crate::game::level::grid::tile::tile_types;
        use crate::game::level::grid::tile::tile_types::TileType;
        use crate::game::level::map::connector::{ConnectorPool, ConnectorRoom};
        use crate::game::level::map::room::{ConnectionFacing, ConnectionType, RoomConnection, RoomDefinition, RoomLayout, RoomRegistryContext, RoomType};
        use super::*;

        pub(super) fn connector_pool(
            context: &mut RoomRegistryContext,
        ) -> ConnectorPool {
            const SIZE: usize = 7;
            const HALF_SIZE: usize = SIZE / 2;

            let basic_room = |gl: Option<TileType>| {
                let mut layout = [[[gl; SIZE]; SIZE]; 1];

                for x in 0..SIZE {
                    for z in 0..SIZE {
                        if (x == 0 || x == SIZE - 1
                            || z == 0 || z == SIZE - 1)
                            && x != HALF_SIZE
                            && z != HALF_SIZE
                        {
                            layout[0][z][x] = None;
                        }
                    }
                }
                
                layout
            };
            
            let basic_connections = vec![
                RoomConnection::new(
                    IVec3::new(HALF_SIZE as i32, 0, 0).into(),
                    ConnectionType::Small,
                    ConnectionFacing::North
                ),
                RoomConnection::new(
                    IVec3::new(SIZE as i32, 0, HALF_SIZE as i32).into(),
                    ConnectionType::Small,
                    ConnectionFacing::East
                ),
                RoomConnection::new(
                    IVec3::new(HALF_SIZE as i32, 0, SIZE as i32).into(),
                    ConnectionType::Small,
                    ConnectionFacing::South
                ),
                RoomConnection::new(
                    IVec3::new(0, 0, HALF_SIZE as i32).into(),
                    ConnectionType::Small,
                    ConnectionFacing::West
                ),
            ];

            let grass_layer = Some(tile_types::grass::LAYER);
            let grass_room = RoomDefinition::new(
                RoomType::Connector,
                basic_connections.clone(),
                Box::new(RoomLayout::new(basic_room(grass_layer))),
                context,
            );
            let grass_room = ConnectorRoom::new(grass_room, 1.0);

            let plank_layer = Some(tile_types::grass::LAYER);
            let plank_room = RoomDefinition::new(
                RoomType::Connector,
                basic_connections.clone(),
                Box::new(RoomLayout::new(basic_room(plank_layer))),
                context,
            );
            let plank_room = ConnectorRoom::new(plank_room, 1.0);

            ConnectorPool(vec![
                grass_room,
                plank_room,
            ])
        }
    }
}