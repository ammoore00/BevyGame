use crate::game::level::map::room::RoomResource;
use bevy::prelude::*;
use data::prelude::*;

pub(super) fn plugin(_app: &mut App) {}

#[derive(Debug)]
pub struct _Connector {}

/// Pool of rooms, which can be selected from for a transition
#[derive(Debug, Clone, Default)]
pub struct TransitionRoomPool(pub Vec<TransitionRoom>);

/// Definition for rooms to be used for populating transition sections
#[derive(Debug, Clone)]
pub struct TransitionRoom {
    /// The physical room layout
    room: ResourceLocation<RoomResource>,
    /// The weight for how often this room will appear
    _weight: f32,
    /// The rule for how many times this room can appear
    /// - Any: No restrictions on the number of occurrences
    /// - Max: Can appear at most X times
    _rule: TransitionRoomRule,
}

impl TransitionRoom {
    pub fn new(room: ResourceLocation<RoomResource>, weight: f32) -> Self {
        Self {
            room,
            _weight: weight,
            _rule: TransitionRoomRule::Any,
        }
    }

    pub fn _with_rule(room: ResourceLocation<RoomResource>, weight: f32, rule: TransitionRoomRule) -> Self {
        Self { room, _weight: weight, _rule: rule }
    }

    pub fn room(&self) -> &ResourceLocation<RoomResource> {
        &self.room
    }

    pub fn _weight(&self) -> f32 {
        self._weight
    }

    pub fn _rule(&self) -> &TransitionRoomRule {
        &self._rule
    }
}

/// Rules to be used for populating transitions with rooms
#[derive(Debug, Clone, Copy, Reflect)]
pub enum TransitionRoomRule {
    /// Can appear at most X times
    Max(usize),
    /// No restrictions on the number of occurrences
    Any,
}
