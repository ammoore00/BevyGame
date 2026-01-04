use bevy::prelude::*;
use crate::game::level::map::room::RoomDefinition;

pub(super) fn plugin(app: &mut App) {
}

#[derive(Debug)]
pub struct Connector {

}

#[derive(Debug, Reflect)]
pub struct ConnectorPool(pub Vec<ConnectorRoom>);

#[derive(Debug, Reflect)]
pub struct ConnectorRoom {
    room: RoomDefinition,
    weight: f32,
}

impl ConnectorRoom {
    pub fn new(room: RoomDefinition, weight: f32) -> Self {
        Self { room, weight }
    }
    
    pub fn room(&self) -> &RoomDefinition {
        &self.room
    }
    
    pub fn weight(&self) -> f32 {
        self.weight
    }
}