use crate::codec::{
    AttackCodec, AttackSetCodec, ColliderCodec, HealthEventKind, HitboxCodec, KeyFrameCodec,
};
use crate::loader::{LoaderJobManager, RonAssetLoader};
use crate::resource::characters::{AnimationResource, CharacterSpriteResource};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::WorldCoords;
use data::prelude::*;
use data::resource::resource_kind;
use getset::{CopyGetters, Getters};
use physics::ColliderKind;
use std::ops::Deref;
use std::time::Duration;

pub(in crate::resource) fn plugin(app: &mut App) {
    app.init_asset::<AttackDefinition>();
    app.init_asset_loader::<RonAssetLoader<AttackCodec, AttackDefinition>>();
    app.add_registry_with_discovery::<AttackResource>();

    app.init_asset::<AttackSet>();
    app.init_asset_loader::<RonAssetLoader<AttackSetCodec, AttackSet>>();
    app.add_registry_with_discovery::<AttackSetResource>();
}

#[derive(SystemParam)]
pub struct AttackContext<'w> {
    pub attack_registry: SystemRegistry<'w, AttackResource>,
    pub attack_set_registry: SystemRegistry<'w, AttackSetResource>,
    pub animation_context: SystemRegistry<'w, AnimationResource>,
    pub character_sprite_registry: SystemRegistry<'w, CharacterSpriteResource>,
}

#[resource_kind(path = "characters/attacks", asset_kind = AttackDefinition)]
pub struct AttackResource;

#[derive(Debug, Clone, Asset, TypePath, Getters, CopyGetters)]
pub struct AttackDefinition {
    #[getset(get = "pub")]
    duration: Duration,
    #[getset(get_copy = "pub")]
    stamina_cost: usize,
    #[getset(get = "pub")]
    animation: ResourceLocation<AnimationResource>,
    #[getset(get = "pub")]
    particle_sprite: ResourceLocation<CharacterSpriteResource>,

    #[getset(get = "pub")]
    key_frames: KeyFrameList,
    #[getset(get = "pub")]
    exclusion_groups: Vec<ExclusionGroup>,
}
impl AttackDefinition {
    pub fn get_progress_increment(&self, duration: Duration) -> AttackProgress {
        AttackProgress(duration.as_millis() as f32 / self.duration.as_millis() as f32)
    }
}
impl TryFrom<AttackCodec> for AttackDefinition {
    type Error = AttackDefinitionError;

    fn try_from(value: AttackCodec) -> Result<Self, AttackDefinitionError> {
        let key_frames = KeyFrameList::try_from_codec(value.key_frames, value.duration)?;
        let exclusion_groups = key_frames.get_exclusion_groups();

        Ok(AttackDefinition {
            duration: Duration::from_millis(value.duration),
            stamina_cost: value.stamina_cost,
            animation: value.animation,
            particle_sprite: value.particle_sprite,

            key_frames,
            exclusion_groups,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AttackDefinitionError {
    #[error(transparent)]
    KeyFrameList(#[from] KeyFrameListError),
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct AttackProgress(f32);
impl AttackProgress {
    /// Create a new attack progress value. The provided value is clamped between 0 and 1.
    pub fn new(progress: f32) -> Self {
        let error = 0.01;
        let lower_bound = 0.0 - error;
        let upper_bound = 1.0 + error;

        if !(lower_bound..=upper_bound).contains(&progress) {
            warn!("Attack progress value is out of range [0, 1]: {}", progress);
        }

        Self(progress.clamp(0.0, 1.0))
    }
}
impl Deref for AttackProgress {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<AttackProgress> for f32 {
    fn from(value: AttackProgress) -> Self {
        *value
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct FrameProgress(f32);
impl FrameProgress {
    /// Create a new frame progress value. The provided value is clamped between 0 and 1.
    pub fn new(progress: f32) -> Self {
        let error = 0.01;
        let lower_bound = 0.0 - error;
        let upper_bound = 1.0 + error;

        if !(lower_bound..=upper_bound).contains(&progress) {
            warn!("Frame progress value is out of range [0, 1]: {}", progress);
        }

        Self(progress.clamp(0.0, 1.0))
    }
}
impl Deref for FrameProgress {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<FrameProgress> for f32 {
    fn from(value: FrameProgress) -> Self {
        *value
    }
}

#[derive(Debug, Clone, TypePath)]
pub struct KeyFrameList {
    key_frames: Vec<KeyFrame>,
}
impl KeyFrameList {
    /// Get any key frames that are active at the given normalized time.
    ///
    /// Normalized time is a value between 0 and 1,
    /// where 0 is the start of the attack and 1 is the end.
    ///
    /// Note that this is different from the normalized time used for hitbox interpolation.
    pub fn get_active_frames(&self, attack_progress: AttackProgress) -> Vec<&KeyFrame> {
        self.key_frames
            .iter()
            .filter(|key_frame| key_frame.current_progress(attack_progress).is_some())
            .collect()
    }

    /// Gets the key frame at the given index.
    ///
    /// This should only be used when storing an index from an existing key frame
    /// for later retrieval and should not be used without prior reference.
    pub fn get_key_frame(&self, index: usize) -> Option<&KeyFrame> {
        self.key_frames.get(index)
    }

    fn get_exclusion_groups(&self) -> Vec<ExclusionGroup> {
        self.key_frames
            .iter()
            .map(|key_frame| key_frame.exclusion_group.clone())
            .collect()
    }

    fn try_from_codec(
        codecs: Vec<KeyFrameCodec>,
        attack_duration: u64,
    ) -> Result<Self, KeyFrameListError> {
        let key_frames = codecs
            .iter()
            .enumerate()
            .map(|(index, codec)| {
                let start = codec.start_time as f32 / attack_duration as f32;
                let end = codec.end_time as f32 / attack_duration as f32;

                let start = AttackProgress::new(start);
                let end = AttackProgress::new(end);

                let hitbox = codec.hitbox.clone().try_into()?;

                let disable_on_hit_iframes = codec.disable_on_hit_iframes.unwrap_or(false);

                let exclusion_group = codec
                    .exclusion_group
                    .as_ref()
                    .map(|group_name| ExclusionGroup::Named(group_name.clone()))
                    .unwrap_or(ExclusionGroup::Indexed(index));

                Ok(KeyFrame {
                    active_frames: [start, end],
                    hitbox,

                    health_event: codec.health_event,
                    disable_on_hit_iframes,

                    index,
                    exclusion_group,
                })
            })
            .collect::<Result<Vec<_>, HitboxError>>()?;

        Ok(Self::from(key_frames))
    }
}
impl From<Vec<KeyFrame>> for KeyFrameList {
    fn from(key_frames: Vec<KeyFrame>) -> Self {
        Self { key_frames }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyFrameListError {
    #[error(transparent)]
    Hitbox(#[from] HitboxError),
}

#[derive(Debug, Clone, TypePath, Getters)]
pub struct KeyFrame {
    /// Start and end time as normalized [0, 1] proportional timings
    active_frames: [AttackProgress; 2],
    hitbox: Hitbox,

    // TODO: Extra properties like damage, knockback, etc.
    #[getset(get = "pub")]
    disable_on_hit_iframes: bool,
    #[getset(get = "pub")]
    health_event: HealthEventKind,

    /// Unique identifier within this attack for this keyframe
    #[getset(get = "pub")]
    index: usize,
    /// Exclusion group for hitboxes which count together
    #[getset(get = "pub")]
    exclusion_group: ExclusionGroup,
}
impl KeyFrame {
    /// Get how far along in this keyframe the attack is.
    ///
    /// Returns None if the attack time is outside the keyframe's active range.
    pub fn current_progress(&self, attack_progress: AttackProgress) -> Option<FrameProgress> {
        let normalized_start_time = *self.active_frames[0];
        let normalized_end_time = *self.active_frames[1];

        if !(normalized_start_time..=normalized_end_time).contains(&attack_progress) {
            return None;
        }

        let length = normalized_end_time - normalized_start_time;
        if length < f32::EPSILON {
            return Some(FrameProgress::new(0.0));
        }

        let delta = *attack_progress - normalized_start_time;
        Some(FrameProgress::new(delta / length))
    }

    /// Get the hitbox data for the current frame progress
    pub fn get_current_interpolated_hitbox(&self, frame_progress: FrameProgress) -> HitboxData {
        self.hitbox.get_current_interpolated_hitbox(frame_progress)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypePath)]
pub enum ExclusionGroup {
    Named(String),
    Indexed(usize),
}

#[derive(Debug, Clone, TypePath)]
pub enum Hitbox {
    Static(HitboxData),
    Interpolated(InterpolatedHitbox),
    Swept(SweptHitbox),
}
impl Hitbox {
    /// Get the hitbox data at the given normalized time.
    ///
    /// Normalized time in this case is used as a proportion between the start and end position
    /// for the attack. The hitbox does not store the actual length of the attack, so this
    /// uses the normalized time to interpolate between the start and end hitbox data.
    pub fn get_current_interpolated_hitbox(&self, frame_progress: FrameProgress) -> HitboxData {
        match self {
            Hitbox::Static(hitbox_data) => hitbox_data.clone(),
            Hitbox::Interpolated(interpolated_hitbox) => {
                interpolated_hitbox.get_current_interpolated_hitbox(frame_progress)
            }
            Hitbox::Swept(swept_hitbox) => {
                swept_hitbox.get_current_interpolated_hitbox(frame_progress)
            }
        }
    }
}
// TODO: Convert this to use TryFrom once fallible type conversion is supported
impl TryFrom<HitboxCodec> for Hitbox {
    type Error = HitboxError;

    fn try_from(value: HitboxCodec) -> Result<Self, Self::Error> {
        let hitbox = match value {
            HitboxCodec::Static { collider, offset } => Hitbox::Static(HitboxData {
                collider,
                offset: offset.into(),
            }),
            HitboxCodec::Interpolated {
                collider_start,
                collider_end,
                offset_start,
                offset_end,
            } => {
                if collider_start.kind() != collider_end.kind() {
                    return Err(HitboxError::ColliderMismatch(
                        collider_start.kind(),
                        collider_end.kind(),
                    ));
                }

                Hitbox::Interpolated(InterpolatedHitbox::new(
                    collider_start,
                    collider_end,
                    offset_start.into(),
                    offset_end.into(),
                ))
            }
            HitboxCodec::Swept { .. } => Hitbox::Swept(SweptHitbox {
                // TODO
            }),
        };
        Ok(hitbox)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HitboxError {
    #[error("Collider kind mismatch: {0:?}, {1:?}")]
    ColliderMismatch(ColliderKind, ColliderKind),
}

/// Data used to describe an instantaneous hitbox for a single frame.
#[derive(Debug, Clone, Getters, TypePath)]
pub struct HitboxData {
    #[getset(get = "pub")]
    collider: ColliderCodec,
    #[getset(get = "pub")]
    offset: WorldCoords,
}

#[derive(Debug, Clone, Getters)]
pub struct InterpolatedHitbox {
    #[getset(get = "pub")]
    _collider_start: ColliderCodec,
    #[getset(get = "pub")]
    _collider_end: ColliderCodec,
    #[getset(get = "pub")]
    _offset_start: WorldCoords,
    #[getset(get = "pub")]
    _offset_end: WorldCoords,
}
impl InterpolatedHitbox {
    fn new(
        collider_start: ColliderCodec,
        collider_end: ColliderCodec,
        offset_start: WorldCoords,
        offset_end: WorldCoords,
    ) -> Self {
        Self {
            _collider_start: collider_start,
            _collider_end: collider_end,
            _offset_start: offset_start,
            _offset_end: offset_end,
        }
    }

    fn get_current_interpolated_hitbox(&self, _frame_progress: FrameProgress) -> HitboxData {
        todo!()
    }
}

#[derive(Debug, Clone, TypePath)]
pub struct SweptHitbox {
    // TODO: Implement
}
impl SweptHitbox {
    fn get_current_interpolated_hitbox(&self, _frame_progress: FrameProgress) -> HitboxData {
        todo!()
    }
}

#[resource_kind(path = "characters/attack_sets", asset_kind = AttackSet)]
pub struct AttackSetResource;

#[derive(Debug, Clone, Asset, TypePath, Getters)]
pub struct AttackSet {
    #[getset(get = "pub")]
    attacks: Vec<ResourceLocation<AttackResource>>,
}
impl AttackSet {
    pub fn iter(&self) -> impl Iterator<Item = &ResourceLocation<AttackResource>> {
        self.attacks.iter()
    }
}
impl From<AttackSetCodec> for AttackSet {
    fn from(value: AttackSetCodec) -> Self {
        AttackSet {
            attacks: value.attacks,
        }
    }
}
