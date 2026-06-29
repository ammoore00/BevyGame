use bevy::prelude::App;

pub mod loader;
pub mod loc;
pub mod prototyping;
pub mod registry;
pub mod resource;
pub mod sprite;

pub mod prelude {
    pub use crate::{
        audio::{AudioRegistry, AudioResource},
        data::{
            loader::{LoaderJobManager, RonAssetLoader},
            loc::{ResourceLocation, loc},
            prototyping::{
                Prototype, PrototypeBuilder, PrototypeFinalizedMarker, PrototypeMarkerToken,
            },
            registry::{ResourceRegistry, SystemRegistry, SystemRegistryMut},
            resource::{ResourceFileType, ResourceKind},
        },
        define_data_resource, define_resource, define_sprite_resource,
        game::{
            character::{
                animation::{AnimationResource, ResolvedAnimationRegistry},
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
