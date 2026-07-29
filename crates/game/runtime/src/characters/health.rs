use assets::codec::{DamageKind, DamageModifierCodec, DamageModifierKind, HealthEventKind};
use bevy::prelude::*;
use common::AppSystems;
use std::collections::HashMap;
use std::time::Duration;
use strum::IntoEnumIterator;
use crate::characters::DeathEvent;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_iframes.in_set(AppSystems::TickTimers));

    app.add_observer(on_health_event);
    app.add_observer(on_add_iframes);
}

#[derive(SceneComponent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[scene(HealthProps)]
pub struct Health {
    pub max: usize,
    pub current: usize,
}
impl Health {
    fn scene(props: HealthProps) -> impl Scene {
        bsn! [
            Health {
                max: {props.max_health},
                current: {props.max_health},
            }
            IFrames
            DamageModifiers::from(props.damage_modifiers)
        ]
    }
}
impl Default for Health {
    fn default() -> Self {
        Self {
            max: 100,
            current: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthProps {
    pub max_health: usize,
    pub damage_modifiers: DamageModifierCodec,
}
impl Default for HealthProps {
    fn default() -> Self {
        Self {
            max_health: 100,
            damage_modifiers: DamageModifierCodec::default(),
        }
    }
}

#[derive(EntityEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HealthEvent {
    entity: Entity,
    event_type: HealthEventKind,
}

impl HealthEvent {
    pub fn new(entity: Entity, event_type: HealthEventKind) -> Self {
        Self { entity, event_type }
    }
}

#[derive(Component, Debug, Clone)]
struct DamageModifiers(HashMap<DamageKind, DamageModifierKind>);
impl Default for DamageModifiers {
    fn default() -> Self {
        let mut map = HashMap::new();

        for kind in DamageKind::iter() {
            map.insert(kind, DamageModifierKind::default());
        }

        Self(map)
    }
}
impl From<DamageModifierCodec> for DamageModifiers {
    fn from(codec: DamageModifierCodec) -> Self {
        let mut map = codec.modifiers;

        for kind in DamageKind::iter() {
            map.entry(kind).or_default();
        }

        Self(map)
    }
}

fn on_health_event(
    event: On<HealthEvent>,
    mut query: Query<(Entity, &mut Health, Option<&DamageModifiers>, Option<&IFrames>)>,
    mut commands: Commands,
) {
    if let Ok((entity, mut health, modifiers, iframes)) = query.get_mut(event.entity) {
        match event.event_type {
            HealthEventKind::Heal(amount) => {
                health.current += amount.min(health.max - health.current)
            }
            HealthEventKind::Damage(amount, damage_type) => {
                if let Some(iframes) = iframes
                    && !iframes.duration.is_zero()
                {
                    // TODO: Fix iframes
                    //return;
                }

                let modifier = if let Some(modifiers) = modifiers {
                    modifiers
                        .0
                        .get(&damage_type)
                        .unwrap_or(&DamageModifierKind::None)
                } else {
                    &DamageModifierKind::None
                };

                health.current -= modifier.apply(amount).min(health.current);
            }
            HealthEventKind::Set(amount) => health.current = amount.clamp(0, health.max),
            HealthEventKind::FullHeal => health.current = health.max,
            HealthEventKind::InstantDeath => health.current = 0,
            HealthEventKind::None => {}
        }

        if health.current == 0 {
            commands.trigger(DeathEvent { entity });
        }
    }
}

#[derive(EntityEvent)]
pub struct AddIFrames {
    entity: Entity,
    duration: Duration,
    mode: IFrameMode,
}
impl AddIFrames {
    pub fn new(entity: Entity, duration: Duration) -> Self {
        Self {
            entity,
            duration,
            mode: IFrameMode::default(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IFrameMode {
    /// Set the duration of the invincibility frames to the provided duration,
    /// unless it is already higher
    #[default]
    Refresh,
    /// Set the duration of the invincibility frames to the provided duration, even if it is higher
    _Set,
    /// Cap the duration of the invincibility frames to the provided duration,
    /// leaving it unchanged if it is already lower
    _Cap,
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
struct IFrames {
    duration: Duration,
}

fn update_iframes(query: Query<&mut IFrames>, time: Res<Time>) {
    for mut iframes in query {
        if !iframes.duration.is_zero() {
            let current = iframes.duration;
            iframes.duration -= time.delta().min(current);
        }
    }
}

fn on_add_iframes(event: On<AddIFrames>, mut query: Query<&mut IFrames>, mut commands: Commands) {
    if let Ok(mut iframes) = query.get_mut(event.entity) {
        match event.mode {
            IFrameMode::Refresh => iframes.duration = iframes.duration.max(event.duration),
            IFrameMode::_Set => iframes.duration = event.duration,
            IFrameMode::_Cap => iframes.duration = iframes.duration.min(event.duration),
        }
        return;
    }

    commands.entity(event.entity).insert(IFrames {
        duration: event.duration,
    });
}
