use std::str::FromStr;

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    AttackAttribute,
    HealthAttribute,
    Attack,
    AttackPercent,
    Damage,
    DamagePercent,
    Health,
    HealthPercent,
    Gold,
    GoldPercent,
    Trumpet,
    TrumpetPercent,
    Level,
    Tier,
    Uses,
    Experience,
    Ailment,
}

impl FromStr for AttributeType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "attack" => AttributeType::Attack,
            "damage" => AttributeType::Damage,
            "health" => AttributeType::Health,
            "gold" => AttributeType::Gold,
            "trumpet" | "trumpets" => AttributeType::Trumpet,
            "level" => AttributeType::Level,
            "tier" => AttributeType::Tier,
            "uses" => AttributeType::Uses,
            "experience" => AttributeType::Experience,
            "ailment" => AttributeType::Ailment,
            _ => bail!("Not a valid AttributeType {s}"),
        })
    }
}

impl AttributeType {
    /// Converts [`AttributeType`] variant to a 'percent' labeled variant.
    /// * ex. [`AttributeType::Gold`] -> [`AttributeType::GoldPercent`]
    pub fn into_percent_variant(self) -> anyhow::Result<Self> {
        Ok(match self {
            AttributeType::Attack => AttributeType::AttackPercent,
            AttributeType::Damage => AttributeType::DamagePercent,
            AttributeType::Health => AttributeType::HealthPercent,
            AttributeType::Gold => AttributeType::GoldPercent,
            AttributeType::Trumpet => AttributeType::TrumpetPercent,
            _ => bail!("{self:?} doesn't have a AttributeType 'percent' variant."),
        })
    }
}
