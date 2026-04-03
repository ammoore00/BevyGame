use bevy_game_2d::data::{ResourceLocation, ResourceType};
use bevy_game_2d::datagen_api::room::{ConnectionFacing, ConnectionSize, RoomCodec, RoomConnection, RoomResource};
use bevy_game_2d::datagen_api::tile::TileResource;
use crate::{create_dir, write_data, WriteError};

pub fn generate_rooms() -> Result<(), WriteError> {
    create_dir(RoomResource::root_dir())?;
    
    create_room_data(basic_room("grass"))?;
    create_room_data(basic_room("planks"))?;
    
    Ok(())
}

fn create_room_data(
    room_data: RoomData,
) -> Result<(), WriteError> {
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
        Self { loc: loc.parse().unwrap(), tile_palette, tiles, connections }
    }
}
impl From<RoomData> for RoomCodec {
    fn from(data: RoomData) -> Self {
        RoomCodec::new(LATEST_FORMAT, data.tile_palette, data.tiles, data.connections)
    }
}

const LATEST_FORMAT: u8 = 1;

fn basic_room(tile: &str) -> RoomData {
    const SIZE: usize = 7;
    const HALF_SIZE: usize = SIZE / 2;

    let mut layout = vec![vec![vec![1; SIZE]; SIZE]];

    for x in 0..SIZE {
        for z in 0..SIZE {
            if (x == 0 || x == SIZE - 1
                || z == 0 || z == SIZE - 1)
                && x != HALF_SIZE
                && z != HALF_SIZE
            {
                layout[0][z][x] = 0;
            }
        }
    }

    let basic_connections = vec![
        RoomConnection::new(
            [HALF_SIZE as i32, 0, 0].into(),
            ConnectionSize::Small,
            ConnectionFacing::North
        ),
        RoomConnection::new(
            [SIZE as i32, 0, HALF_SIZE as i32].into(),
            ConnectionSize::Small,
            ConnectionFacing::East
        ),
        RoomConnection::new(
            [HALF_SIZE as i32, 0, SIZE as i32].into(),
            ConnectionSize::Small,
            ConnectionFacing::South
        ),
        RoomConnection::new(
            [0, 0, HALF_SIZE as i32].into(),
            ConnectionSize::Small,
            ConnectionFacing::West
        ),
    ];
    
    let loc = format!("basic_{}", tile);
    RoomData::new(
        loc.as_str(),
        vec![tile.parse().unwrap()],
        layout,
        basic_connections,
    )
}