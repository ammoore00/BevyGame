use std::array;
use bevy::prelude::*;
use crate::game::level::grid::tile::tile_types;
use crate::game::level::grid::tile::tile_types::TileType;
use crate::game::level::map::transition::{TransitionRoom, TransitionRoomPool};
use crate::game::level::map::{MapDefinition, MapPool, MapType};
use crate::game::level::map::room::{ConnectionFacing, ConnectionSize, RoomConnection, RoomDefinition, RoomLayout, RoomRegistryContext, RoomType};

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<Palette>();
    app.init_resource::<Palettes>();
}

/// Resource holding handles to each palette asset
#[derive(Resource, Asset, Debug, Reflect)]
pub struct Palettes {
    pub standard: Handle<Palette>,
}

impl FromWorld for Palettes {
    fn from_world(world: &mut World) -> Self {
        let mut context = world.resource_mut::<RoomRegistryContext>();
        let standard_palette = StandardPalette::create_palette(&mut context);

        let assets = world.resource::<AssetServer>();

        Self {
            standard: assets.add(standard_palette),
        }
    }
}

#[derive(Asset, Debug, Reflect)]
pub struct Palette {
    main_map_pool: MapPool,
    transition_pool: TransitionRoomPool,
}

impl Palette {
    pub fn transition_pool(&self) -> &TransitionRoomPool {
        &self.transition_pool
    }
    
    pub fn main_map_pool(&self) -> &MapPool {
        &self.main_map_pool
    }
}

trait PaletteDefinition {
    fn create_palette(context: &mut RoomRegistryContext) -> Palette;
    fn create_main_map_pool() -> MapPool;
    fn create_transition_pool(context: &mut RoomRegistryContext) -> TransitionRoomPool;
}

struct StandardPalette;
impl PaletteDefinition for StandardPalette {
    fn create_palette(
        context: &mut RoomRegistryContext,
    ) -> Palette {
        Palette {
            main_map_pool: Self::create_main_map_pool(),
            transition_pool: Self::create_transition_pool(context),
        }
    }

    fn create_main_map_pool() -> MapPool {
        let main_map = MapDefinition {
            map_type: MapType::Main,
            map_size: 3,
        };
        MapPool(vec![main_map])
    }

    fn create_transition_pool(context: &mut RoomRegistryContext) -> TransitionRoomPool {
        const SIZE: usize = 7;
        const HALF_SIZE: usize = SIZE / 2;

        let basic_room = |tile: Option<TileType>| {
            let mut layout: [[[_; SIZE]; SIZE]; 1] = array::from_fn(
                |_| array::from_fn(
                    |_| array::from_fn(
                        |_| tile.clone()
                    )
                )
            );

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
                ConnectionSize::Small,
                ConnectionFacing::North
            ),
            RoomConnection::new(
                IVec3::new(SIZE as i32, 0, HALF_SIZE as i32).into(),
                ConnectionSize::Small,
                ConnectionFacing::East
            ),
            RoomConnection::new(
                IVec3::new(HALF_SIZE as i32, 0, SIZE as i32).into(),
                ConnectionSize::Small,
                ConnectionFacing::South
            ),
            RoomConnection::new(
                IVec3::new(0, 0, HALF_SIZE as i32).into(),
                ConnectionSize::Small,
                ConnectionFacing::West
            ),
        ];

        let grass_layer = Some(tile_types::grass::LAYER);
        let grass_room = RoomDefinition::new(
            RoomType::Transition,
            basic_connections.clone(),
            Box::new(RoomLayout::new(basic_room(grass_layer))),
            context,
        );
        let grass_room = TransitionRoom::new(grass_room, 1.0);

        let plank_layer = Some(tile_types::light_planks::LAYER);
        let plank_room = RoomDefinition::new(
            RoomType::Transition,
            basic_connections.clone(),
            Box::new(RoomLayout::new(basic_room(plank_layer))),
            context,
        );
        let plank_room = TransitionRoom::new(plank_room, 1.0);

        TransitionRoomPool(vec![
            grass_room,
            plank_room,
        ])
    }
}