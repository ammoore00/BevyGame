use bevy::prelude::*;
use common::AppSystems;
use enum_map::{Enum, EnumMap};
use runtime::characters::player::{AttackInputEvent, DodgeInputEvent, JumpInputEvent};
use runtime::prelude::Player;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<InputBuffer>();

    app.add_systems(Update, update_input_buffer.in_set(AppSystems::RecordInput));

    app.add_observer(on_insert_buffer);
    app.add_observer(on_clear_buffer);
}

const INPUT_BUFFER_DURATION: Duration = Duration::from_millis(100);

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputKind {
    Dodge,
    Attack,
    Jump,
}
impl InputKind {
    fn trigger_event(&self, entity: Entity, mut commands: Commands) {
        let mut entity = commands.entity(entity);
        match self {
            InputKind::Dodge => entity.trigger(DodgeInputEvent),
            InputKind::Attack => entity.trigger(AttackInputEvent),
            InputKind::Jump => entity.trigger(JumpInputEvent),
        };
    }
}

#[derive(Clone)]
struct BufferedInput {
    expiration_timestamp: Duration,
}

#[derive(Resource, Default)]
struct InputBuffer {
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
    _input_buffer: ResMut<InputBuffer>,
    _time: Res<Time>,

    mut commands: Commands,
    player: Single<Entity, With<Player>>,
) {
    event.kind.trigger_event(player.entity(), commands.reborrow())
    //input_buffer.inputs[event.kind] = Some(BufferedInput {
    //    expiration_timestamp: time.elapsed() + INPUT_BUFFER_DURATION,
    //});
}

fn on_clear_buffer(event: On<ClearBuffer>, mut input_buffer: ResMut<InputBuffer>) {
    input_buffer.inputs[event.kind] = None;
}
