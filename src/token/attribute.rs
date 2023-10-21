use std::str::FromStr;

use anyhow::bail;

use super::ParseNumber;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityType {
    // Numerical values.
    Pet(Option<i32>),
    Food(Option<i32>),
    Toy(Option<i32>),
    Perk(Option<i32>),
    Ailment(Option<i32>),

    Space(Option<i32>),
    /// Phase of battle.
    Battle(Option<i32>),
    Turn(Option<i32>),

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

impl EntityType {
    pub(crate) fn value(&self) -> Option<i32> {
        match self {
            EntityType::Attack(v)
            | EntityType::Damage(v)
            | EntityType::Health(v)
            | EntityType::Gold(v)
            | EntityType::Trumpet(v)
            | EntityType::Level(v)
            | EntityType::Tier(v)
            | EntityType::Uses(v)
            | EntityType::Pet(v)
            | EntityType::Food(v)
            | EntityType::Toy(v)
            | EntityType::Perk(v)
            | EntityType::Ailment(v)
            | EntityType::Space(v)
            | EntityType::Turn(v)
            | EntityType::Battle(v)
            | EntityType::Experience(v) => *v,
            EntityType::AttackPercent(v)
            | EntityType::HealthPercent(v)
            | EntityType::DamagePercent(v)
            | EntityType::GoldPercent(v)
            | EntityType::TrumpetPercent(v) => v.map(|val| val as i32),
        }
    }
}

impl ParseNumber for EntityType {
    fn parse_num_str(&mut self, num_str: &str) -> anyhow::Result<&mut Self> {
        let cleaned_num_str = num_str.trim_start_matches('+');
        match self {
            EntityType::Attack(ref mut v)
            | EntityType::Damage(ref mut v)
            | EntityType::Health(ref mut v)
            | EntityType::Gold(ref mut v)
            | EntityType::Trumpet(ref mut v)
            | EntityType::Level(ref mut v)
            | EntityType::Tier(ref mut v)
            | EntityType::Uses(ref mut v)
            | EntityType::Pet(ref mut v)
            | EntityType::Food(ref mut v)
            | EntityType::Toy(ref mut v)
            | EntityType::Perk(ref mut v)
            | EntityType::Ailment(ref mut v)
            | EntityType::Space(ref mut v)
            | EntityType::Turn(ref mut v)
            | EntityType::Battle(ref mut v)
            | EntityType::Experience(ref mut v) => {
                v.replace(cleaned_num_str.parse()?);
            }
            EntityType::AttackPercent(ref mut v)
            | EntityType::HealthPercent(ref mut v)
            | EntityType::DamagePercent(ref mut v)
            | EntityType::GoldPercent(ref mut v)
            | EntityType::TrumpetPercent(ref mut v) => {
                v.replace(cleaned_num_str.parse()?);
            }
        }

        Ok(self)
    }
}

impl FromStr for EntityType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "pet" | "pets" => EntityType::Pet(None),
            "food" | "foods" => EntityType::Food(None),
            "toy" | "toys" => EntityType::Toy(None),
            "perk" | "perks" => EntityType::Perk(None),
            "ailment" | "ailments" => EntityType::Ailment(None),
            "turn" | "turns" => EntityType::Turn(None),
            "battle" | "battles" => EntityType::Battle(None),
            "space" => EntityType::Space(None),
            "attack" => EntityType::Attack(None),
            "damage" => EntityType::Damage(None),
            "health" => EntityType::Health(None),
            "gold" => EntityType::Gold(None),
            "trumpet" | "trumpets" => EntityType::Trumpet(None),
            "level" => EntityType::Level(None),
            "tier" => EntityType::Tier(None),
            "uses" => EntityType::Uses(None),
            "experience" => EntityType::Experience(None),
            _ => bail!("Not a valid EntityType {s}"),
        })
    }
}

impl EntityType {
    /// Converts [`EntityType`] variant to a 'percent' labeled variant.
    /// * ex. [`EntityType::Gold`] -> [`EntityType::GoldPercent`]
    pub fn into_percent_variant(self) -> anyhow::Result<Self> {
        Ok(match self {
            EntityType::Attack(val) => EntityType::AttackPercent(val.map(|v| v as f32)),
            EntityType::Damage(val) => EntityType::DamagePercent(val.map(|v| v as f32)),
            EntityType::Health(val) => EntityType::HealthPercent(val.map(|v| v as f32)),
            EntityType::Gold(val) => EntityType::GoldPercent(val.map(|v| v as f32)),
            EntityType::Trumpet(val) => EntityType::TrumpetPercent(val.map(|v| v as f32)),
            _ => bail!("{self:?} doesn't have a EntityType 'percent' variant."),
        })
    }
}
