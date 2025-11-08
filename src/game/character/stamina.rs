use crate::screens::Screen;
use crate::{AppSystems, PausableSystems};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_stamina_timer.in_set(AppSystems::TickTimers),
            update_stamina.in_set(AppSystems::Update),
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    )
    .add_observer(on_stamina_event);
}

#[derive(Component)]
pub struct Stamina {
    pub max: usize,
    pub current: usize,
    pub regen_rate: usize,
    pub regen_delay: f32,
    pub regen_timer: Timer,
    pub regen_delay_timer: Option<Timer>,
}

impl Stamina {
    pub fn new(max: usize, regen_per_second: usize, regen_delay: f32) -> Self {
        let regen_interval = 0.1;
        let regen_rate = (regen_per_second as f32 * regen_interval).max(0.0) as usize;

        Self {
            max,
            current: max,
            regen_rate,
            regen_delay,
            regen_timer: Timer::from_seconds(regen_interval, TimerMode::Repeating),
            regen_delay_timer: None,
        }
    }
}

#[derive(EntityEvent)]
pub struct StaminaEvent {
    entity: Entity,
    cost: usize,
}

impl StaminaEvent {
    pub fn new(entity: Entity, cost: usize) -> Self {
        Self { entity, cost }
    }
}

fn on_stamina_event(event: On<StaminaEvent>, mut query: Query<&mut Stamina>) {
    if let Ok(mut stamina) = query.get_mut(event.entity) {
        stamina.current = (stamina.current as isize - event.cost as isize).max(0) as usize;

        let regen_delay = if stamina.current > 0 {
            stamina.regen_delay
        } else {
            stamina.regen_delay * 2.0
        };

        stamina.regen_delay_timer = Some(Timer::from_seconds(regen_delay, TimerMode::Once));
    }
}

fn update_stamina_timer(time: Res<Time>, mut query: Query<&mut Stamina>) {
    for mut stamina in &mut query {
        stamina.regen_timer.tick(time.delta());
        if let Some(ref mut timer) = stamina.regen_delay_timer {
            timer.tick(time.delta());

            if timer.is_finished() {
                stamina.regen_delay_timer = None;
            }
        }
    }
}

fn update_stamina(mut query: Query<&mut Stamina>) {
    for mut stamina in &mut query {
        if stamina.regen_delay_timer.is_some() {
            continue;
        }

        if stamina.regen_timer.is_finished() {
            stamina.current += stamina.regen_rate;
            stamina.current = stamina.current.min(stamina.max);
        }
    }
}
