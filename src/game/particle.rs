use crate::{AppSystems, PausableSystems};
use bevy::prelude::*;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            update_particle.in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    )
    .add_observer(on_particle_spawn_event);
}

fn update_animation_timer(
    time: Res<Time>,
    mut query: Query<(Entity, &mut ParticleAnimation), With<Particle>>,
    mut commands: Commands,
) {
    for (particle, mut animation) in &mut query {
        animation.update_timer(time.delta());

        if animation.has_expired() {
            commands.entity(particle).despawn();
        }
    }
}

fn update_particle(mut query: Query<(&ParticleAnimation, &mut Sprite), With<Particle>>) {
    for (animation, mut sprite) in &mut query {
        let Some(ref mut atlas) = sprite.texture_atlas else {
            continue;
        };

        atlas.index = animation.start_index + animation.current_frame;
    }
}

#[derive(Component, Clone, Debug)]
pub struct Particle;

#[derive(Event, Clone, Debug)]
pub struct ParticleSpawnEvent {
    sprite: Sprite,
    particle_animation: ParticleAnimation,
    parent: Option<Entity>,
}

impl ParticleSpawnEvent {
    pub fn _new(sprite: Sprite, particle_animation: ParticleAnimation) -> Self {
        Self {
            sprite,
            particle_animation,
            parent: None,
        }
    }

    pub fn with_parent(
        sprite: Sprite,
        particle_animation: ParticleAnimation,
        parent: Entity,
    ) -> Self {
        Self {
            sprite,
            particle_animation,
            parent: Some(parent),
        }
    }
}

fn on_particle_spawn_event(event: On<ParticleSpawnEvent>, mut commands: Commands) {
    let particle = commands
        .spawn((
            Particle,
            event.sprite.clone(),
            event.particle_animation.clone(),
            Transform::from_translation(Vec3::Z * 100.0),
        ))
        .id();

    if let Some(parent) = event.parent {
        commands.entity(parent).add_child(particle);
    }
}

#[derive(Component, Clone, Debug)]
pub struct ParticleAnimation {
    timer: Timer,
    start_index: usize,
    num_frames: usize,
    current_frame: usize,
}

impl ParticleAnimation {
    pub fn new(start_index: usize, num_frames: usize, interval: Duration) -> Self {
        Self {
            timer: Timer::new(interval, TimerMode::Repeating),
            start_index,
            num_frames,
            current_frame: 0,
        }
    }

    fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);

        if !self.timer.is_finished() {
            return;
        }

        self.current_frame += 1;
    }

    fn has_expired(&self) -> bool {
        self.current_frame >= self.num_frames
    }
}
