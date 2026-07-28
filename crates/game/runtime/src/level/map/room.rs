use crate::level::grid;
use crate::level::grid::TileMap;
use crate::level::grid::tile::tile;
use assets::resource::level::{
    RoomDefinition, RoomLayout, RoomRegistry, TileAsset, TileLayout, TileRegistry,
    TileSpriteRegistry,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::{Scale, TileCoords};

pub fn build_room(room: &RoomDefinition, builder_context: &mut RoomBuilderContext) -> TileMap {
    build_layout(room.layout(), builder_context)
}

pub fn build_layout(layout: &RoomLayout, context: &mut RoomBuilderContext) -> TileMap {
    let tile_map = grid::tile_map();

    for y in 0..layout.bounds().y {
        for z in 0..layout.bounds().z {
            for x in 0..layout.bounds().x {
                let Some(tile_type) = layout.tiles()[layout.index_of([x, y, z])].clone() else {
                    continue;
                };

                let coords = TileCoords(IVec3::new(x as i32, y as i32, z as i32));

                let tile = context
                    .commands
                    .spawn(tile(
                        context.tile_registry.as_ref(),
                        context.tile_assets.as_ref(),
                        context.sprite_registry.as_ref(),
                        &tile_type,
                        coords,
                        context.tile_layout.as_ref(),
                    ))
                    .id();

                tile_map.write().unwrap().insert(coords, tile);
            }
        }
    }

    tile_map
}

/// Context holding references to data necessary to build rooms from their definitions
#[derive(SystemParam)]
pub struct RoomBuilderContext<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub scale: Res<'w, Scale>,
    pub tile_layout: Res<'w, TileLayout>,
    pub tile_registry: Res<'w, TileRegistry>,
    pub tile_assets: Res<'w, Assets<TileAsset>>,
    pub sprite_registry: Res<'w, TileSpriteRegistry>,
    pub room_registry: Res<'w, RoomRegistry>,
    pub room_assets: Res<'w, Assets<RoomDefinition>>,
}
