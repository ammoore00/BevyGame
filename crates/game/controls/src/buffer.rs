use bevy::prelude::*;
use common::AppSystems;
use enum_map::{Enum, EnumMap};
use getset::Getters;
use runtime::characters::player::{AttackInput, DodgeInput, JumpInput};
use runtime::prelude::Player;
use std::marker::ConstParamTy;
use std::time::Duration;
use strum_macros::Display;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<InputBuffer>();

    app.add_systems(Update, update_input_buffer.in_set(AppSystems::ProcessInput));

    app.add_observer(on_insert_buffer);
    app.add_observer(on_clear_buffer);
}

const INPUT_BUFFER_DURATION: Duration = Duration::from_millis(100);

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq, Hash, ConstParamTy, Display)]
pub enum InputKind {
    Dodge,
    Attack,
    Jump,
}
impl InputKind {
    fn trigger_event(&self, entity: Entity, mut commands: Commands) {
        let mut entity = commands.entity(entity);
        match self {
            InputKind::Dodge => entity.trigger(DodgeInput),
            InputKind::Attack => entity.trigger(AttackInput),
            InputKind::Jump => entity.trigger(JumpInput),
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Getters)]
pub struct BufferedInput {
    #[getset(get = "pub")]
    expiration_timestamp: Duration,
}

#[derive(Resource, Default, Getters)]
pub struct InputBuffer {
    #[getset(get = "pub")]
    inputs: EnumMap<InputKind, Option<BufferedInput>>,
}

/// For each input in the buffer, trigger the event for it every frame
/// until either the event succeeds or the input expires
fn update_input_buffer(
    player: Single<Entity, With<Player>>,
    input_buffer: Res<InputBuffer>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (kind, input) in input_buffer.inputs.iter() {
        let Some(input) = input else {
            continue;
        };

        if input.expiration_timestamp < time.elapsed() {
            commands.trigger(ClearBuffer { kind });
            continue;
        }

        kind.trigger_event(player.entity(), commands.reborrow());
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Hash, derive_new::new)]
pub struct InsertBuffer {
    kind: InputKind,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Hash, derive_new::new)]
pub struct ClearBuffer {
    kind: InputKind,
}

fn on_insert_buffer(
    event: On<InsertBuffer>,
    mut input_buffer: ResMut<InputBuffer>,
    time: Res<Time>,
) {
    input_buffer.inputs[event.kind] = Some(BufferedInput {
        expiration_timestamp: time.elapsed() + INPUT_BUFFER_DURATION,
    });
}

fn on_clear_buffer(event: On<ClearBuffer>, mut input_buffer: ResMut<InputBuffer>) {
    input_buffer.inputs[event.kind] = None;
}

pub fn on_input_success<E: EventPattern, const KIND: InputKind>(_: On<E>, mut commands: Commands) {
    commands.trigger(ClearBuffer::new(KIND));
}
