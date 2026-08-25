use crate::{WriteError, create_dir, write_data};
use assets::codec::RoomCodec;
use assets::resource::level::{
    ConnectionFacing, ConnectionSize, RoomConnection, RoomResource, TileResource,
};
use data::prelude::*;

pub fn generate_rooms() -> Result<(), WriteError> {
    create_dir(RoomResource::ROOT_DIR)?;

    create_room_data(basic_room())?;

    Ok(())
}

fn create_room_data(room_data: RoomData) -> Result<(), WriteError> {
    let loc = room_data.loc.clone();
    let codec = RoomCodec::from(room_data);
    write_data(loc, &codec)
}

struct RoomData {
    loc: ResourceLocation<RoomResource>,
    tile_palette: Vec<ResourceLocation<TileResource>>,
    /// Stored in YZX order (outer to inner)
    tiles: Vec<Vec<Vec<u8>>>,
    connections: Vec<RoomConnection>,
}
impl RoomData {
    fn new(
        loc: &str,
        tile_palette: Vec<ResourceLocation<TileResource>>,
        tiles: Vec<Vec<Vec<u8>>>,
        connections: Vec<RoomConnection>,
    ) -> Self {
        
        
        Self {
            loc: loc.parse().unwrap(),
            tile_palette,
            tiles,
            connections,
        }
    }
}
impl From<RoomData> for RoomCodec {
    fn from(data: RoomData) -> Self {
        RoomCodec::new(
            LATEST_FORMAT,
            data.tile_palette,
            data.tiles,
            data.connections,
        )
    }
}

const LATEST_FORMAT: u8 = 1;

fn basic_room() -> RoomData {
    const SIZE: usize = 7;
    const HALF_SIZE: usize = SIZE / 2;
    const HEIGHT: usize = 2;

    let layout = vec![
        vec![0,  0,  0,  17, 0,  0,  0],
        vec![0,  12, 13, 28, 13, 14, 0],
        vec![0,  11, 28, 28, 28, 4,  0],
        vec![19, 28, 28, 28, 28, 28, 19],
        vec![0,  11, 28, 28, 28, 4,  0],
        vec![0,  10, 8,  28, 8,  6,  0],
        vec![0,  0,  0,  17, 0,  0,  0],
    ];

    let basic_connections = vec![
        RoomConnection::new(
            [HALF_SIZE as i32, 0, 0].into(),
            ConnectionSize::Small,
            ConnectionFacing::North,
        ),
        RoomConnection::new(
            [SIZE as i32, 0, HALF_SIZE as i32].into(),
            ConnectionSize::Small,
            ConnectionFacing::East,
        ),
        RoomConnection::new(
            [HALF_SIZE as i32, 0, SIZE as i32].into(),
            ConnectionSize::Small,
            ConnectionFacing::South,
        ),
        RoomConnection::new(
            [0, 0, HALF_SIZE as i32].into(),
            ConnectionSize::Small,
            ConnectionFacing::West,
        ),
    ];

    use crate::tiles::*;
    
    let loc = "basic_grass";
    RoomData::new(
        loc,
        vec![
            GRASS_ALL_OUTLINE.parse().unwrap(),
            GRASS_ALL.parse().unwrap(),
            GRASS_S_OUTLINE.parse().unwrap(),
            GRASS_S.parse().unwrap(),
            GRASS_SW_OUTLINE.parse().unwrap(),
            GRASS_SW.parse().unwrap(),
            GRASS_W_OUTLINE.parse().unwrap(),
            GRASS_W.parse().unwrap(),
            GRASS_NW_OUTLINE.parse().unwrap(),
            GRASS_NW.parse().unwrap(),
            GRASS_N.parse().unwrap(),
            GRASS_NE.parse().unwrap(),
            GRASS_E.parse().unwrap(),
            GRASS_SE_OUTLINE.parse().unwrap(),
            GRASS_SE.parse().unwrap(),
            GRASS_NS_OUTLINE.parse().unwrap(),
            GRASS_NS.parse().unwrap(),
            GRASS_EW_OUTLINE.parse().unwrap(),
            GRASS_EW.parse().unwrap(),
            GRASS_NOT_S_OUTLINE.parse().unwrap(),
            GRASS_NOT_S.parse().unwrap(),
            GRASS_NOT_W_OUTLINE.parse().unwrap(),
            GRASS_NOT_W.parse().unwrap(),
            GRASS_NOT_N_OUTLINE.parse().unwrap(),
            GRASS_NOT_N.parse().unwrap(),
            GRASS_NOT_E_OUTLINE.parse().unwrap(),
            GRASS_NOT_E.parse().unwrap(),
            GRASS.parse().unwrap(),
        ],
        vec![layout],
        basic_connections,
    )
}
