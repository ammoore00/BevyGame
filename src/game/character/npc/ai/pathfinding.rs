use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {

}

pub(super) fn pathfinder_bundle() -> impl Bundle {
    (
        Pathfinder,
    )
}

#[derive(Component, Debug, Clone)]
struct Pathfinder;