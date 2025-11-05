use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_health_event);
}

#[derive(Component, Asset, Clone, Reflect)]
pub struct Health {
    pub max: usize,
    pub current: usize,
}

impl Health {
    pub fn new(max: usize) -> Self {
        Self { max, current: max }
    }

    pub fn with_current(max: usize, current: usize) -> Self {
        let current = current.clamp(0, max);
        Self { max, current }
    }
}

#[derive(EntityEvent)]
pub struct HealthEvent {
    entity: Entity,
    event_type: HealthEventType,
}

impl HealthEvent {
    pub fn new(entity: Entity, event_type: HealthEventType) -> Self {
        Self { entity, event_type }
    }
}

pub enum HealthEventType {
    Heal(usize),
    Damage(usize, DamageType),
    Set(usize),
    FullHeal,
    InstantDeath,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DamageType {
    Generic,
    Fall,

    Slash,
    Blunt,
    Pierce,

    Shock,
    Fire,
    Void,
    Explosive,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DamageModifier {
    None,
    Vulnerability(f32),
    Resistance(f32),
    Immunity,
}

fn on_health_event(event: On<HealthEvent>, mut query: Query<(&mut Health)>) {
    if let Ok(mut health) = query.get_mut(event.entity) {
        match event.event_type {
            HealthEventType::Heal(amount) => {
                health.current += amount.min(health.max - health.current)
            }
            HealthEventType::Damage(amount, _) => health.current -= amount.min(health.current),
            HealthEventType::Set(amount) => health.current = amount.clamp(0, health.max),
            HealthEventType::FullHeal => health.current = health.max,
            HealthEventType::InstantDeath => health.current = 0,
        }
    }
}
