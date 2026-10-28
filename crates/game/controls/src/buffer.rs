use std::sync::Arc;
use bevy::prelude::*;
use enum_map::{Enum, EnumMap};
use std::time::Duration;
use common::AppSystems;

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

type InputEvent = dyn Fn(Commands) + Send + Sync;

#[derive(Clone)]
struct BufferedInput {
    event: Arc<InputEvent>,
    expiration_timestamp: Duration,
}

#[derive(Resource, Default)]
struct InputBuffer {
    inputs: EnumMap<InputKind, Option<BufferedInput>>,
}

/// For each input in the buffer, trigger the event for it every frame until either the event succeeds or the input expires
fn update_input_buffer(
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
        
        (input.event)(commands.reborrow());
    }
}

#[derive(Event, Clone)]
pub struct InsertBuffer {
    kind: InputKind,
    event: Arc<InputEvent>
}

#[derive(Event, Debug, Clone)]
pub struct ClearBuffer {
    kind: InputKind,
}

fn on_insert_buffer(
    event: On<InsertBuffer>,
    mut input_buffer: ResMut<InputBuffer>,
    time: Res<Time>,
) {
    input_buffer.inputs[event.kind] = Some(BufferedInput {
        event: event.event.clone(),
        expiration_timestamp: time.elapsed() + INPUT_BUFFER_DURATION,
    });
}

fn on_clear_buffer(
    event: On<ClearBuffer>,
    mut input_buffer: ResMut<InputBuffer>,
) {
    input_buffer.inputs[event.kind] = None;
}
