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

    let mut layout = vec![vec![vec![0; SIZE]; SIZE]; HEIGHT];
    layout[0] = vec![vec![1; SIZE]; SIZE];
    
    let cracks = [
        (1, 1),
        (2, 4),
        (3, 6),
        (4, 2),
        (6, 3),
    ];

    for x in 0..SIZE {
        for z in 0..SIZE {
            if (x == 0 || x == SIZE - 1 || z == 0 || z == SIZE - 1)
                && x != HALF_SIZE
                && z != HALF_SIZE
            {
                let index = if cracks.contains(&(x, z)) {
                    1
                } else {
                    0
                };
                
                layout[0][z][x] = index;
            }
        }
    }

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

    let loc = "basic_tiles";
    RoomData::new(
        loc,
        vec![
            "tile".parse().unwrap(),
            "tile_cracked".parse().unwrap(),
        ],
        layout,
        basic_connections,
    )
}
