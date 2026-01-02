use bevy::prelude::*;
use crate::game::level::map::connector::ConnectorPool;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Palettes>();

    app.add_plugins((standard::plugin,));
}

#[derive(Resource, Asset, Debug, Reflect)]
pub struct Palettes {
    standard: Handle<Palette>,
}

impl FromWorld for Palettes {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            standard: assets.add(standard::palette(assets)),
        }
    }
}

#[derive(Asset, Debug, Reflect)]
pub struct Palette {
    connector_pool: Handle<ConnectorPool>,
}

mod standard {
    use super::*;

    pub(super) fn plugin(app: &mut App) {
        app.add_plugins((connectors::plugin,));
    }
    
    pub(super) fn palette(assets: &AssetServer) -> Palette {
        Palette {
            connector_pool: assets.add(connectors::connector_pool(assets)),
        }
    }

    mod connectors {
        use crate::game::level::map::connector::ConnectorPool;
        use super::*;

        pub(super) fn plugin(app: &mut App) {}
        
        pub(super) fn connector_pool(assets: &AssetServer) -> ConnectorPool {
            todo!()
        }
    }
}