use bevy::prelude::*;
use common::marker;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_particle_spawn_event);
}

marker!(Particle);

#[derive(Event, Clone, Debug)]
pub struct ParticleSpawnEvent {
    sprite: Sprite,
    parent: Option<Entity>,
}

fn on_particle_spawn_event(event: On<ParticleSpawnEvent>, mut commands: Commands) {
    let particle = commands
        .spawn((
            Particle,
            event.sprite.clone(),
            Transform::from_translation(Vec3::Z * 100.0),
        ))
        .id();

    if let Some(parent) = event.parent {
        commands.entity(parent).add_child(particle);
    }
}
