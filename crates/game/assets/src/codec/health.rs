use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DamageModifierCodec {
    pub format: u8,
    pub modifiers: HashMap<DamageKind, DamageModifierKind>,
}
impl DamageModifierCodec {
    pub const LATEST_FORMAT: u8 = 1;
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
    pub fn apply(&self, amount: usize) -> usize {
        match self {
            DamageModifierKind::None => amount,
            DamageModifierKind::Vulnerability(tier) => (amount as f32 * tier.as_f32()) as usize,
            DamageModifierKind::Resistance(tier) => (amount as f32 / tier.as_f32()) as usize,
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
pub enum DamageKind {
    Generic,

    Slash,
    Blunt,
    Pierce,

    Shock,
    Fire,
    Void,
    Explosive,
    Corrosive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthEventKind {
    Heal(usize),
    Damage(usize, DamageKind),
    Set(usize),
    FullHeal,
    InstantDeath,
    None,
}
