//! Player-specific behavior.

use crate::characters::health::Health;
use crate::characters::stamina::Stamina;
use crate::characters::CharacterPrototype;
use assets::resource::characters::{CharacterResource, CharacterSpriteResource};
use bevy::ecs::template::OptionTemplate;
use bevy::image::TextureAtlasTemplate;
use bevy::prelude::*;
use common::Facing;
use data::prelude::*;

mod input;

pub use input::{AimInputEvent, AttackInputEvent, JumpInputEvent, MoveInputEvent};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(input::plugin);
}

pub(crate) fn player(position: Vec3) -> impl Scene {
    bsn! [
        @Player {
            @position,
            @max_speed: 4.5,
        }
    ]
}

#[derive(Debug, Default, Clone)]
pub struct PlayerProps {
    position: Vec3,
    max_speed: f32,
}

#[derive(SceneComponent, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
#[scene(PlayerProps)]
pub struct Player;
impl Player {
    fn scene(props: PlayerProps) -> impl Scene {
        bsn! [
            #Player
            Player
            @CharacterPrototype {
                @position: {props.position},
                @max_speed: {props.max_speed},
                @data_loc: {loc::<CharacterResource>("player").unwrap()}
            }
            Health::new(300)
            Stamina::new(200, 200, 1.0)
            Children [
                #IndicatorRing
                AimFacing
                Visibility::Hidden
                Transform::from_translation(Vec3::new(0.0, 0.0, 100.0))
                Sprite {
                    image: {loc::<CharacterSpriteResource>("player/indicator_ring").unwrap()},
                    texture_atlas: OptionTemplate::Some(TextureAtlasTemplate {
                        layout: asset_value(TextureAtlasLayout::from_grid(UVec2::splat(64), 8, 1, None, None)),
                        index: 0,
                    }),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.25),
                }
            ]
        ]
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Eq)]
pub struct AimFacing(pub Option<Facing>);