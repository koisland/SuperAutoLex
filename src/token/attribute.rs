use std::str::FromStr;

use anyhow::bail;

use super::ParseNumber;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttributeType {
    // Attribute
    AttackAttribute,
    HealthAttribute,
    Ailment,

    // Numerical values.
    Attack(Option<i32>),
    Damage(Option<i32>),
    Health(Option<i32>),
    Gold(Option<i32>),
    Trumpet(Option<i32>),
    Level(Option<i32>),
    Tier(Option<i32>),
    Uses(Option<i32>),
    Experience(Option<i32>),

    // Percents
    AttackPercent(Option<f32>),
    HealthPercent(Option<f32>),
    DamagePercent(Option<f32>),
    GoldPercent(Option<f32>),
    TrumpetPercent(Option<f32>),
}

impl ParseNumber for AttributeType {
    fn parse_num_str(&mut self, num_str: &str) -> anyhow::Result<&mut Self> {
        let cleaned_num_str = num_str.trim_start_matches('+');
        match self {
            AttributeType::Attack(ref mut v)
            | AttributeType::Damage(ref mut v)
            | AttributeType::Health(ref mut v)
            | AttributeType::Gold(ref mut v)
            | AttributeType::Trumpet(ref mut v)
            | AttributeType::Level(ref mut v)
            | AttributeType::Tier(ref mut v)
            | AttributeType::Uses(ref mut v)
            | AttributeType::Experience(ref mut v) => {
                v.replace(cleaned_num_str.parse()?);
            }
            AttributeType::AttackPercent(ref mut v)
            | AttributeType::HealthPercent(ref mut v)
            | AttributeType::DamagePercent(ref mut v)
            | AttributeType::GoldPercent(ref mut v)
            | AttributeType::TrumpetPercent(ref mut v) => {
                v.replace(cleaned_num_str.parse()?);
            }
            _ => bail!("Cannot add value to attribute."),
        }

        Ok(self)
    }
}

impl FromStr for AttributeType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "attack" => AttributeType::Attack(None),
            "damage" => AttributeType::Damage(None),
            "health" => AttributeType::Health(None),
            "gold" => AttributeType::Gold(None),
            "trumpet" | "trumpets" => AttributeType::Trumpet(None),
            "level" => AttributeType::Level(None),
            "tier" => AttributeType::Tier(None),
            "uses" => AttributeType::Uses(None),
            "experience" => AttributeType::Experience(None),
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
            AttributeType::Attack(val) => AttributeType::AttackPercent(val.map(|v| v as f32)),
            AttributeType::Damage(val) => AttributeType::DamagePercent(val.map(|v| v as f32)),
            AttributeType::Health(val) => AttributeType::HealthPercent(val.map(|v| v as f32)),
            AttributeType::Gold(val) => AttributeType::GoldPercent(val.map(|v| v as f32)),
            AttributeType::Trumpet(val) => AttributeType::TrumpetPercent(val.map(|v| v as f32)),
            _ => bail!("{self:?} doesn't have a AttributeType 'percent' variant."),
        })
    }
}
