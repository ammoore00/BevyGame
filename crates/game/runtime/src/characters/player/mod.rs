//! Player-specific behavior.

use crate::characters::CharacterPrototype;
use crate::characters::health::Health;
use crate::characters::stamina::Stamina;
use assets::action_states::Dodging;
use assets::resource::characters::{CharacterResource, CharacterSpriteResource};
use bevy::ecs::template::OptionTemplate;
use bevy::image::TextureAtlasTemplate;
use bevy::prelude::*;
use common::{Facing, marker};
use data::prelude::*;
use std::f32::consts::PI;

mod input;

pub use input::{
    AimInputEvent, AttackInput, AttackInputConfirmed, DodgeInput, DodgeInputConfirmed, JumpInput,
    JumpInputConfirmed, MoveInputEvent,
};
use physics::{ApplyForce, Force, RemoveForce};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(input::plugin);

    app.add_systems(FixedUpdate, apply_dodge_movement);
}

pub(crate) fn player(position: Vec3) -> impl Scene {
    bsn! {
        @Player {
            @position,
            @max_speed: 4.5,
        }
    }
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
        bsn! {
            #Player
            Player
            @CharacterPrototype {
                @position: {props.position},
                @max_speed: {props.max_speed},
                @data_loc: {loc::<CharacterResource>("player").unwrap()}
            }
            @Health {
                @max_health: 300
            }
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
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Eq)]
pub struct AimFacing(pub Option<Facing>);

marker!(PlayerDodgeForce);

const DODGE_SPEED_START: f32 = 1.0;
const DODGE_SPEED_PEAK: f32 = 7.0;

fn apply_dodge_movement(
    player: Single<(Entity, Option<&Dodging>), With<Player>>,
    mut commands: Commands,
) {
    let (player, dodging) = player.into_inner();

    if let Some(dodge_state) = dodging {
        let ease_multiplier = (dodge_state.completion() * PI).sin();
        let speed = DODGE_SPEED_START + ease_multiplier * (DODGE_SPEED_PEAK - DODGE_SPEED_START);

        let target_velocity = dodge_state.direction() * speed;
        let force = Force::target_velocity(target_velocity);
        commands.trigger(ApplyForce::new::<PlayerDodgeForce>(player, force));
    } else {
        commands.trigger(RemoveForce::new::<PlayerDodgeForce>(player));
    }
}
