use crate::game::grid::coords::rotate_movement_to_screen_space;
use crate::game::physics::movement::MovementController;
use crate::{AppSystems, PausableSystems};
use bevy::prelude::*;
use std::fmt::Debug;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (update_animation_state, update_animation_atlas)
                .chain()
                .in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

fn update_animation_timer(time: Res<Time>, mut query: Query<&mut CharacterAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

fn update_animation_state(query: Query<(&mut CharacterAnimation, &MovementController)>) {
    for (mut animation, controller) in query {
        //let direction = rotate_movement_to_screen_space(controller.intent);
        let direction = controller.intent;

        // Calculate angle in radians (-PI to PI)
        // Note: atan2(z, x) where x is "forward" and z is "right"
        let angle = direction.x.atan2(direction.z);

        // Convert to 0-8 range, where each direction occupies 45 degrees (PI/4 radians)
        // Add PI to shift range from [-PI, PI] to [0, 2*PI]
        // Add PI/8 to center the divisions on the cardinal directions
        // Divide by PI/4 (45 degrees) to get 0-8 range
        let direction_index = ((angle + std::f32::consts::PI + std::f32::consts::FRAC_PI_8 + std::f32::consts::FRAC_PI_2 * 3.0)
            / std::f32::consts::FRAC_PI_4)
            .floor() as i32
            % 8;

        animation.facing = Facing::from(direction_index as usize);

        if controller.intent.length() >= 0.8 {
            animation
                .set_running()
                .unwrap_or_else(|_| animation.set_idle());
        } else if controller.intent.length() > 1e-6 {
            animation
                .set_walking()
                .unwrap_or_else(|_| animation.set_idle());
        } else {
            animation.set_idle();
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

#[derive(Debug, Clone, Copy, Reflect)]
enum Facing {
    NorthWest = 0,
    West = 1,
    SouthWest = 2,
    South = 3,
    SouthEast = 4,
    East = 5,
    NorthEast = 6,
    North = 7,
}

impl From<usize> for Facing {
    fn from(index: usize) -> Self {
        match index {
            0 => Self::NorthWest,
            1 => Self::West,
            2 => Self::SouthWest,
            3 => Self::South,
            4 => Self::SouthEast,
            5 => Self::East,
            6 => Self::NorthEast,
            7 => Self::North,
            _ => unreachable!(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum AnimationError {
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
        Sprite::from_atlas_image(
            self.get_image().clone(),
            self.get_atlas().clone(),
        )
    }

    fn set_idle(&mut self) {
        if matches!(self.state, AnimationState::Idling) {
            return;
        }

        self.state = AnimationState::Idling;
        self.timer = Timer::new(self.capabilities.idle.interval, TimerMode::Repeating);
        self.frame = 0;
    }

    fn set_walking(&mut self) -> Result<(), AnimationError> {
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

    fn set_running(&mut self) -> Result<(), AnimationError> {
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

    fn reset(&mut self) {
        self.state = AnimationState::Idling;
        self.timer = Timer::new(self.capabilities.idle.interval, TimerMode::Repeating);
        self.frame = 0;
    }

    fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);

        println!("{:?}", self.timer.elapsed());

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
                    if let Some(walk) = &self.capabilities.walk {
                        walk.frames
                    } else {
                        // If we somehow got into an invalid state, reset the animation to idle
                        self.reset();
                        return;
                    }
                }
            };

        println!("{:?}", self.frame);
    }

    fn get_image(&self) -> &Handle<Image> {
        let default = &self.capabilities.idle.image;

        match self.state {
            AnimationState::Idling => default,
            AnimationState::Walking => {
                if let Some(walk) = &self.capabilities.walk {
                    &walk.image
                } else {
                    default
                }
            }
            AnimationState::Running => {
                if let Some(run) = &self.capabilities.run {
                    &run.image
                } else {
                    default
                }
            }
        }
    }

    fn get_atlas(&self) -> &TextureAtlas {
        let default = &self.capabilities.idle.atlas;

        match self.state {
            AnimationState::Idling => default,
            AnimationState::Walking => {
                if let Some(walk) = &self.capabilities.walk {
                    &walk.atlas
                } else {
                    default
                }
            }
            AnimationState::Running => {
                if let Some(run) = &self.capabilities.run {
                    &run.atlas
                } else {
                    default
                }
            }
        }
    }

    fn get_atlas_index(&self) -> usize {
        let default = self.frame;

        match self.state {
            AnimationState::Idling => {
                let offset = self.facing as usize * self.capabilities.idle.frames;
                default + offset
            },
            AnimationState::Walking => {
                if let Some(walk) = &self.capabilities.walk {
                    let offset = self.facing as usize * walk.frames;
                    self.frame + offset
                } else {
                    default
                }
            }
            AnimationState::Running => {
                if let Some(run) = &self.capabilities.run {
                    let offset = self.facing as usize * run.frames;
                    self.frame + offset
                } else {
                    default
                }
            }
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
}

#[derive(Debug, Clone, Reflect)]
pub enum AnimationState {
    Idling,
    Walking,
    Running,
}
