use crate::room::{RoomData, create_room_data};
use crate::{WriteError, create_dir};
use assets::resource::level::{ConnectionFacing, ConnectionSize, RoomConnection, RoomResource};
use data::prelude::ResourceKind;

pub fn generate_rooms() -> Result<(), WriteError> {
    create_dir(RoomResource::ROOT_DIR)?;

    create_room_data(basic_room())?;

    Ok(())
}

fn basic_room() -> RoomData {
    const SIZE: usize = 7;
    const HALF_SIZE: usize = SIZE / 2;
    const HEIGHT: usize = 2;

    let layout = vec![
        vec![0, 0, 0, 16, 0, 0, 0],
        vec![0, 13, 8, 28, 8, 14, 0],
        vec![0, 7, 28, 28, 28, 3, 0],
        vec![18, 28, 28, 28, 28, 28, 18],
        vec![0, 7, 28, 28, 28, 3, 0],
        vec![0, 11, 5, 28, 5, 9, 0],
        vec![0, 0, 0, 16, 0, 0, 0],
    ];

    let layout = vec![
        vec![0, 0, 0, 0, 0, 7, 28, 3, 0, 0, 0, 0, 0],
        vec![0, 13, 8, 8, 8, 28, 28, 28, 8, 8, 8, 14, 0],
        vec![0, 7, 28, 28, 28, 28, 28, 28, 28, 28, 28, 3, 0],
        vec![0, 7, 28, 28, 28, 28, 28, 28, 28, 28, 28, 3, 0],
        vec![0, 7, 28, 28, 28, 28, 28, 28, 28, 28, 28, 3, 0],
        vec![8, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 8],
        vec![28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28],
        vec![5, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 5],
        vec![0, 7, 28, 28, 28, 28, 28, 28, 28, 28, 28, 3, 0],
        vec![0, 7, 28, 28, 28, 28, 28, 28, 28, 28, 28, 3, 0],
        vec![0, 7, 28, 28, 28, 28, 28, 28, 28, 28, 28, 3, 0],
        vec![0, 11, 5, 5, 5, 28, 28, 28, 5, 5, 5, 9, 0],
        vec![0, 0, 0, 0, 0, 7, 28, 3, 0, 0, 0, 0, 0],
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

    use crate::defs::tiles::*;

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
