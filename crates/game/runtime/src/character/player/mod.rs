//! Player-specific behavior.

use crate::character::health::Health;
use crate::character::stamina::Stamina;
use crate::character::CharacterPrototype;
use assets::resource::character::{CharacterResource, CharacterSpriteResource};
use bevy::ecs::template::OptionTemplate;
use bevy::image::TextureAtlasTemplate;
use bevy::prelude::*;
use common::Facing;
use data::prelude::*;

mod input;

pub use input::{InputAttackEvent, InputJumpEvent, InputMoveEvent};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(input::plugin);

    app.add_observer(on_aim_facing);
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

#[derive(EntityEvent, Debug, Clone, derive_new::new)]
pub struct AimFacingEvent {
    entity: Entity,
    facing: Option<Facing>,
}

fn on_aim_facing(
    event: On<AimFacingEvent>,
    mut query: Query<(&mut AimFacing, &mut Sprite, &mut Visibility, &ChildOf)>,
) {
    let Ok((mut aim_facing, mut sprite, mut visibility, child_of)) = query.single_mut() else {
        error!("Failed to get aim facing query!");
        return;
    };

    if child_of.0 != event.entity {
        error!("Aim facing event received for wrong entity!");
        return;
    }

    if event.facing == aim_facing.0 {
        return;
    }

    if let Some(new_facing) = event.facing {
        aim_facing.0 = Some(new_facing);
        visibility
            .set(Box::new(Visibility::Inherited))
            .expect("Failed to set visibility");
        sprite.texture_atlas.as_mut().unwrap().index = new_facing as usize;
    } else {
        aim_facing.0 = None;
        visibility
            .set(Box::new(Visibility::Hidden))
            .expect("Failed to set visibility");
    }
    info!("Aim facing event success!");
}
