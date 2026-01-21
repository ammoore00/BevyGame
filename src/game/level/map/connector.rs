use crate::game::level::map::room::RoomDefinition;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {}

#[derive(Debug)]
pub struct Connector {}

#[derive(Debug, Reflect)]
pub struct ConnectorPool(pub Vec<ConnectorRoom>);

#[derive(Debug, Reflect)]
pub struct ConnectorRoom {
    room: RoomDefinition,
    weight: f32,
    rule: ConnectorRoomRule,
}

impl ConnectorRoom {
    pub fn new(room: RoomDefinition, weight: f32) -> Self {
        Self {
            room,
            weight,
            rule: ConnectorRoomRule::Any,
        }
    }

    pub fn with_rule(room: RoomDefinition, weight: f32, rule: ConnectorRoomRule) -> Self {
        Self { room, weight, rule }
    }

    pub fn room(&self) -> &RoomDefinition {
        &self.room
    }

    pub fn weight(&self) -> f32 {
        self.weight
    }

    pub fn rule(&self) -> &ConnectorRoomRule {
        &self.rule
    }
}

/// Rules to be used for populating connectors with rooms
#[derive(Debug, Clone, Copy, Reflect)]
pub enum ConnectorRoomRule {
    /// Can appear at most X times
    Max(usize),
    /// No restrictions on the number of occurrences
    Any,
}
