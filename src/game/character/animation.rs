use crate::game::character::{CharacterStateTracker, Facing};
use crate::screens::Screen;
use crate::{AppSystems, PausableSystems};
use bevy::prelude::*;
use std::fmt::Debug;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            update_animation_atlas.in_set(AppSystems::Update),
            // Exclusive system cannot be chained
            update_animation_state.in_set(AppSystems::Respond),
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

fn update_animation_timer(time: Res<Time>, mut query: Query<&mut CharacterAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

fn update_animation_state(world: &mut World) {
    // 1. Manually fetch the entities and their data using a query
    // We collect into a Vec to avoid borrowing issues while we use 'world' inside the loop
    let mut query = world.query::<(Entity, &CharacterStateTracker, &Facing)>();
    let entities: Vec<(Entity, CharacterStateTracker, Facing)> = query
        .iter(world)
        .map(|(e, t, f)| (e, t.clone(), *f))
        .collect();

    for (entity, state_tracker, facing) in entities {
        // 2. Get the state using your exclusive helper
        if let Some(state) = crate::game::character::get_state(entity, &state_tracker, world) {
            // 3. Apply the animation updates
            if let Some(mut animation) = world.get_mut::<CharacterAnimation>(entity) {
                animation.facing = facing;
                state.set_animation(&mut animation);
            }
        }
    }
}

fn update_animation_atlas(query: Query<(&CharacterAnimation, &mut Sprite)>) {
    for (animation, ref mut sprite) in query {
        if animation.changed() {
            sprite.image = animation.get_image().clone();

            let mut atlas = animation.get_atlas().clone();
            atlas.index = animation.get_atlas_index();
            sprite.texture_atlas = Some(atlas);
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AnimationError {
    #[error("No animation capability found for state {:?}", .0)]
    NoSuchCapability(AnimationState),
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct CharacterAnimation {
    capabilities: AnimationCapabilities,
    state: AnimationState,
    facing: Facing,
    timer: Timer,
    frame: usize,
}

impl CharacterAnimation {
    pub fn new(capabilities: AnimationCapabilities) -> Self {
        Self {
            capabilities: capabilities.clone(),
            state: AnimationState::Idling,
            facing: Facing::North,
            timer: Timer::new(capabilities.idle.interval, TimerMode::Repeating),
            frame: 0,
        }
    }

    pub fn default_sprite(&self) -> Sprite {
        Sprite::from_atlas_image(self.get_image().clone(), self.get_atlas().clone())
    }

    pub fn set_idle(&mut self) {
        if matches!(self.state, AnimationState::Idling) {
            return;
        }

        self.state = AnimationState::Idling;
        self.timer = Timer::new(self.capabilities.idle.interval, TimerMode::Repeating);
        self.frame = 0;
    }

    pub fn set_walking(&mut self) -> Result<(), AnimationError> {
        if matches!(self.state, AnimationState::Walking) {
            return Ok(());
        }

        let walk = self
            .capabilities
            .walk
            .as_ref()
            .ok_or(AnimationError::NoSuchCapability(AnimationState::Walking))?;

        self.state = AnimationState::Walking;
        self.timer = Timer::new(walk.interval, TimerMode::Repeating);
        self.frame = 0;
        Ok(())
    }

    pub fn set_running(&mut self) -> Result<(), AnimationError> {
        if matches!(self.state, AnimationState::Running) {
            return Ok(());
        }

        let run = self
            .capabilities
            .run
            .as_ref()
            .ok_or(AnimationError::NoSuchCapability(AnimationState::Running))?;

        self.state = AnimationState::Running;
        self.timer = Timer::new(run.interval, TimerMode::Repeating);
        self.frame = 0;
        Ok(())
    }

    pub fn set_sprinting(&mut self) -> Result<(), AnimationError> {
        if matches!(self.state, AnimationState::Sprinting) {
            return Ok(());
        }

        let sprint = self
            .capabilities
            .sprint
            .as_ref()
            .ok_or(AnimationError::NoSuchCapability(AnimationState::Sprinting))?;

        self.state = AnimationState::Sprinting;
        self.timer = Timer::new(sprint.interval, TimerMode::Repeating);
        self.frame = 0;
        Ok(())
    }

    pub fn set_attacking(&mut self) -> Result<(), AnimationError> {
        if matches!(self.state, AnimationState::Attacking) {
            return Ok(());
        }

        let attack = self
            .capabilities
            .attack
            .as_ref()
            .ok_or(AnimationError::NoSuchCapability(AnimationState::Attacking))?;

        self.state = AnimationState::Attacking;
        self.timer = Timer::new(attack.interval, TimerMode::Repeating);
        self.frame = 0;
        Ok(())
    }

    fn reset(&mut self) {
        self.state = AnimationState::Idling;
        self.timer = Timer::new(self.capabilities.idle.interval, TimerMode::Repeating);
        self.frame = 0;
    }

    fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);

        if !self.timer.is_finished() {
            return;
        }

        self.frame = (self.frame + 1)
            % match self.state {
                AnimationState::Idling => self.capabilities.idle.frames,
                AnimationState::Walking => {
                    if let Some(walk) = &self.capabilities.walk {
                        walk.frames
                    } else {
                        // If we somehow got into an invalid state, reset the animation to idle
                        self.reset();
                        return;
                    }
                }
                AnimationState::Running => {
                    if let Some(run) = &self.capabilities.run {
                        run.frames
                    } else {
                        // If we somehow got into an invalid state, reset the animation to idle
                        self.reset();
                        return;
                    }
                }
                AnimationState::Sprinting => {
                    if let Some(sprint) = &self.capabilities.sprint {
                        sprint.frames
                    } else {
                        // If we somehow got into an invalid state, reset the animation to idle
                        self.reset();
                        return;
                    }
                }
                AnimationState::Attacking => {
                    if let Some(attack) = &self.capabilities.attack {
                        attack.frames
                    } else {
                        // If we somehow got into an invalid state, reset the animation to idle
                        self.reset();
                        return;
                    }
                }
            };
    }

    fn get_image(&self) -> Handle<Image> {
        let default = self.capabilities.idle.image.clone();

        let maybe_image = |capability: Option<&CharacterAnimationData>| {
            capability
                .map(|data| data.image.clone())
                .unwrap_or(default.clone())
        };

        match self.state {
            AnimationState::Idling => default,
            AnimationState::Walking => maybe_image(self.capabilities.walk.as_ref()),
            AnimationState::Running => maybe_image(self.capabilities.run.as_ref()),
            AnimationState::Sprinting => maybe_image(self.capabilities.sprint.as_ref()),
            AnimationState::Attacking => maybe_image(self.capabilities.attack.as_ref()),
        }
    }

    fn get_atlas<'a>(&'a self) -> &'a TextureAtlas {
        let default = &self.capabilities.idle.atlas;

        let maybe_atlas = |capability: Option<&'a CharacterAnimationData>| {
            capability.map(|data| &data.atlas).unwrap_or(default)
        };

        match self.state {
            AnimationState::Idling => default,
            AnimationState::Walking => maybe_atlas(self.capabilities.walk.as_ref()),
            AnimationState::Running => maybe_atlas(self.capabilities.run.as_ref()),
            AnimationState::Sprinting => maybe_atlas(self.capabilities.sprint.as_ref()),
            AnimationState::Attacking => maybe_atlas(self.capabilities.attack.as_ref()),
        }
    }

    fn get_atlas_index(&self) -> usize {
        let default = self.frame;

        let maybe_offset = |capability: Option<&CharacterAnimationData>| {
            capability
                .map(|data| self.frame + self.facing as usize * data.frames)
                .unwrap_or(default)
        };

        match self.state {
            AnimationState::Idling => {
                let offset = self.facing as usize * self.capabilities.idle.frames;
                default + offset
            }
            AnimationState::Walking => maybe_offset(self.capabilities.walk.as_ref()),
            AnimationState::Running => maybe_offset(self.capabilities.run.as_ref()),
            AnimationState::Sprinting => maybe_offset(self.capabilities.sprint.as_ref()),
            AnimationState::Attacking => maybe_offset(self.capabilities.attack.as_ref()),
        }
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct CharacterAnimationData {
    pub image: Handle<Image>,
    pub atlas: TextureAtlas,
    pub frames: usize,
    pub interval: Duration,
}

#[derive(Debug, Clone, Reflect)]
pub struct AnimationCapabilities {
    pub idle: CharacterAnimationData,
    pub walk: Option<CharacterAnimationData>,
    pub run: Option<CharacterAnimationData>,
    pub sprint: Option<CharacterAnimationData>,
    pub attack: Option<CharacterAnimationData>,
}

#[derive(Debug, Clone, Reflect)]
pub enum AnimationState {
    Idling,
    Walking,
    Running,
    Sprinting,
    Attacking,
}
