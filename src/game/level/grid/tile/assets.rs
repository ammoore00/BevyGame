use std::sync::LazyLock;
use bevy::prelude::*;
use crate::StartupSystems;

pub(in crate::game) fn plugin(app: &mut App) {
    app.add_systems(
        Startup,
        populate_tile_assets.in_set(StartupSystems::PopulateAssets)
    );
}

static TILE_SPRITE_LAYOUT: LazyLock<TextureAtlasLayout> = LazyLock::new(|| TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 8, Some(UVec2::splat(1)), None));

fn populate_tile_assets(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let tile_assets = TileLayout {
        layout: asset_server.add(TILE_SPRITE_LAYOUT.clone()),
    };

    commands.insert_resource(tile_assets);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct TileLayout {
    //#[dependency]
    layout: Handle<TextureAtlasLayout>,
}

impl TileLayout {
    pub fn layout(&self) -> Handle<TextureAtlasLayout> {
        self.layout.clone()
    }
}
