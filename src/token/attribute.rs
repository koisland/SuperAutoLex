use std::str::FromStr;

use anyhow::bail;

use super::ParseNumber;

#[derive(Debug, Clone, PartialEq)]
pub enum EntityType<'src> {
    // Numerical values.
    Pet {
        number: Option<i32>,
        name: Option<&'src str>,
        attr: Option<&'src str>,
    },
    Food {
        number: Option<i32>,
        name: Option<&'src str>,
    },
    Toy {
        number: Option<i32>,
        name: Option<&'src str>,
    },
    Ability {
        name: Option<&'src str>,
    },
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

impl<'src> EntityType<'src> {
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
            | EntityType::Pet { number: v, .. }
            | EntityType::Food { number: v, .. }
            | EntityType::Toy { number: v, .. }
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
            EntityType::Ability { .. } => None,
        }
    }
}

impl<'src> ParseNumber for EntityType<'src> {
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
            | EntityType::Pet {
                number: ref mut v, ..
            }
            | EntityType::Food {
                number: ref mut v, ..
            }
            | EntityType::Toy {
                number: ref mut v, ..
            }
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
            EntityType::Ability { .. } => {}
        }

        Ok(self)
    }
}

impl<'src> FromStr for EntityType<'src> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "pet" | "pets" => EntityType::Pet {
                number: None,
                name: None,
                attr: None,
            },
            "food" | "foods" => EntityType::Food {
                number: None,
                name: None,
            },
            "toy" | "toys" => EntityType::Toy {
                number: None,
                name: None,
            },
            "perk" | "perks" => EntityType::Perk(None),
            "ailment" | "ailments" => EntityType::Ailment(None),
            "turn" | "turns" => EntityType::Turn(None),
            "battle" | "battles" => EntityType::Battle(None),
            "space" => EntityType::Space(None),
            "attack" => EntityType::Attack(None),
            "damage" => EntityType::Damage(None),
            "health" | "healthy" => EntityType::Health(None),
            "gold" => EntityType::Gold(None),
            "trumpet" | "trumpets" => EntityType::Trumpet(None),
            "level" => EntityType::Level(None),
            "tier" => EntityType::Tier(None),
            "uses" => EntityType::Uses(None),
            "experience" => EntityType::Experience(None),
            "ability" => EntityType::Ability { name: None },
            _ => bail!("Not a valid EntityType {s}"),
        })
    }
}

impl<'src> EntityType<'src> {
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
