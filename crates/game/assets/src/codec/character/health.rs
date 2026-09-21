use crate::loader::Maybe;
use bevy::prelude::TypePath;
use maybe_fields::maybe_fields;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[maybe_fields]
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypePath)]
pub struct HealthCodec {
    pub max_health: u32,
    /// Set of modifiers per damage type.
    ///
    /// Optional - defaults to no modifiers.
    pub damage_modifiers: Maybe<DamageModifierCodec>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DamageModifierCodec {
    pub modifiers: HashMap<DamageKindOrCategory, DamageModifierKind>,
}

#[derive(Default, Debug, Clone)]
pub struct DamageModifierList {
    list: HashMap<DamageKindOrCategory, DamageModifierKind>
}
impl DamageModifierList {
    pub fn get(&self, key: DamageKind) -> Option<&DamageModifierKind> {
        self.list.get(&key.into()).or_else(|| self.list.get(&key.category().into()))
    }
}
impl From<HashMap<DamageKindOrCategory, DamageModifierKind>> for DamageModifierList {
    fn from(list: HashMap<DamageKindOrCategory, DamageModifierKind>) -> Self {
        Self { list }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DamageKindOrCategory {
    Kind(DamageKind),
    Category(DamageCategory),
}
impl From<DamageKind> for DamageKindOrCategory {
    fn from(kind: DamageKind) -> Self {
        DamageKindOrCategory::Kind(kind)
    }
}
impl From<DamageCategory> for DamageKindOrCategory {
    fn from(category: DamageCategory) -> Self {
        DamageKindOrCategory::Category(category)
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageModifierKind {
    #[default]
    None,
    Vulnerability(ModifierTier),
    Resistance(ModifierTier),
    Immunity,
}
impl DamageModifierKind {
    pub fn apply(&self, amount: u32) -> u32 {
        match self {
            DamageModifierKind::None => amount,
            DamageModifierKind::Vulnerability(tier) => (amount as f32 * tier.as_f32()) as u32,
            DamageModifierKind::Resistance(tier) => (amount as f32 / tier.as_f32()) as u32,
            DamageModifierKind::Immunity => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierTier {
    None = 0,
    Small = 10,
    Medium = 20,
    Large = 40,
    Extreme = 80,
}
impl ModifierTier {
    fn as_f32(self) -> f32 {
        f32::from(self)
    }
}
impl From<ModifierTier> for f32 {
    fn from(tier: ModifierTier) -> f32 {
        tier as usize as f32 / 100_f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::EnumIter, Serialize, Deserialize)]
pub enum DamageCategory {
    Generic,
    Physical,
    Elemental,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::EnumIter, Serialize, Deserialize)]
pub enum DamageKind {
    /// Generic damage is used for damage events that cannot be reduced in any way.
    ///
    /// Characters should not be given resistance to generic damage, as it will do nothing.
    Generic,

    // Physical damage types
    Slash,
    Blunt,
    Pierce,
    Explosive,

    // Elemental damage types
    Shock,
    Fire,
    Void,
    Corrosive,
}
impl DamageKind {
    pub fn category(&self) -> DamageCategory {
        match self {
            DamageKind::Generic => DamageCategory::Generic,
            DamageKind::Slash | DamageKind::Blunt | DamageKind::Pierce | DamageKind::Explosive => {
                DamageCategory::Physical
            }
            _ => DamageCategory::Elemental,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthEventKind {
    Heal(u32),
    Damage(u32, DamageKind),
    Set(u32),
    FullHeal,
    InstantDeath,
    None,
}
