use bevy::prelude::*;
use enum_map::{Enum, EnumMap};
use std::time::Duration;
use common::AppSystems;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<InputBuffer>();
    
    app.add_systems(Update, update_input_buffer.in_set(AppSystems::RecordInput));
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputKind {
    Dodge,
    Attack,
    Jump,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BufferedInput {
    kind: InputKind,
    expiration_timestamp: Duration,
}

#[derive(Resource, Default)]
struct InputBuffer {
    inputs: EnumMap<InputKind, Option<BufferedInput>>,
}

fn update_input_buffer(
    mut input_buffer: ResMut<InputBuffer>,
    time: Res<Time>,
) {
    for (_, input) in input_buffer.inputs {
        if let Some(input) = input {
            if input.expiration_timestamp < time.elapsed() {
                input_buffer.inputs[input.kind] = None;
            }
        }
    }
}
