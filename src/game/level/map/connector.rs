use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<ConnectorPool>();
}

#[derive(Debug)]
pub struct Connector {

}

#[derive(Asset, Debug, Reflect)]
pub struct ConnectorPool(pub Vec<ConnectorRoom>);

#[derive(Debug, Reflect)]
pub struct ConnectorRoom {
    room_id: usize,
    weight: f32,
}