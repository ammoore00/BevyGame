use bevy::prelude::*;
use crate::game::level::map::connector::ConnectorPool;
use crate::game::level::map::room::RoomRegistryContext;

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<Palette>();
    app.init_resource::<Palettes>();

    app.add_plugins((standard::plugin,));
}

#[derive(Resource, Asset, Debug, Reflect)]
pub struct Palettes {
    pub standard: Handle<Palette>,
}

impl FromWorld for Palettes {
    fn from_world(world: &mut World) -> Self {
        let mut context = world.resource_mut::<RoomRegistryContext>();
        let palette = standard::palette(&mut context);

        let assets = world.resource::<AssetServer>();

        Self {
            standard: assets.add(palette),
        }
    }
}

#[derive(Asset, Debug, Reflect)]
pub struct Palette {
    connector_pool: ConnectorPool,
}

impl Palette {
    pub fn connector_pool(&self) -> &ConnectorPool {
        &self.connector_pool
    }
}

mod standard {
    use crate::game::level::map::room::RoomRegistryContext;
    use super::*;

    pub(super) fn plugin(app: &mut App) {
        app.add_plugins((connectors::plugin,));
    }

    pub(super) fn palette(
        context: &mut RoomRegistryContext,
    ) -> Palette {
        Palette {
            connector_pool: connectors::connector_pool(context),
        }
    }

    mod connectors {
        use crate::game::level::grid::tile::tile_types;
        use crate::game::level::map::connector::{ConnectorPool, ConnectorRoom};
        use crate::game::level::map::room::{RoomDefinition, RoomLayout, RoomRegistryContext, RoomType};
        use super::*;

        pub(super) fn plugin(app: &mut App) {}

        pub(super) fn connector_pool(
            context: &mut RoomRegistryContext,
        ) -> ConnectorPool {
            let gl = Some(tile_types::grass::LAYER);
            let layout = [[[gl; 16]; 16]; 1];

            let room = RoomDefinition::new(
                RoomType::Connector,
                vec![],
                Box::new(RoomLayout::new(layout)),
                context,
            );

            let connector = ConnectorRoom::new(room, 1.0);

            ConnectorPool(vec![connector])
        }
    }
}