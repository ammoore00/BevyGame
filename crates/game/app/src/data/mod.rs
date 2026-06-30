use bevy::prelude::App;

pub mod loader;

pub mod prelude {
    pub use crate::{
        audio::{AudioRegistry, AudioResource},
        game::{
            character::{
                animation::{AnimationResource, AnimationRegistry},
                assets::{
                    CharacterRegistry, CharacterResource, CharacterSpriteRegistry,
                    CharacterSpriteResource,
                },
                attack::{AttackRegistry, AttackResource, AttackSetRegistry, AttackSetResource},
            },
            level::{
                grid::tile::assets::{
                    TileRegistry, TileResource, TileSpriteRegistry, TileSpriteResource,
                },
                map::{
                    MapRegistry, MapResource,
                    room::{RoomRegistry, RoomResource},
                },
            },
        },
    };
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((loader::plugin,));
}
