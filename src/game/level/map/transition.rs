use crate::game::level::map::room::RoomDefinition;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {}

#[derive(Debug)]
pub struct Connector {}

/// Pool of rooms, which can be selected from for a transition
pub struct TransitionRoomPool(pub Vec<TransitionRoom>);

/// Definition for rooms to be used for populating transition sections
pub struct TransitionRoom {
    /// The physical room layout
    room: RoomDefinition,
    /// The weight for how often this room will appear
    weight: f32,
    /// The rule for how many times this room can appear
    /// - Any: No restrictions on the number of occurrences
    /// - Max: Can appear at most X times
    rule: TransitionRoomRule,
}

impl TransitionRoom {
    pub fn new(room: RoomDefinition, weight: f32) -> Self {
        Self {
            room,
            weight,
            rule: TransitionRoomRule::Any,
        }
    }

    pub fn with_rule(room: RoomDefinition, weight: f32, rule: TransitionRoomRule) -> Self {
        Self { room, weight, rule }
    }

    pub fn room(&self) -> &RoomDefinition {
        &self.room
    }

    pub fn weight(&self) -> f32 {
        self.weight
    }

    pub fn rule(&self) -> &TransitionRoomRule {
        &self.rule
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
