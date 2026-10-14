use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    
}

#[derive(Component)]
#[relationship_target(relationship = SensedBy)]
pub struct SensedTargets(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = SensedTargets)]
pub struct SensedBy(pub Entity);