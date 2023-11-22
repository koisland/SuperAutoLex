//! SAP item attribute/entity tokens.

use std::{borrow::Cow, str::FromStr};

use anyhow::bail;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::ParseNumber;

/// All possible entity types in Super Auto Pets.
/// - If [`None`], the entity itself.
///     - ex. `EntityType::Battle(None)` -> `battle`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EntityType<'src> {
    /// Pet.
    Pet {
        /// Specific pet name.
        name: Option<Cow<'src, str>>,
        /// Specific pet attribute.
        /// - ex. `Strawberry`
        attr: Option<Cow<'src, str>>,
        /// Specific pet pack.
        pack: Option<Cow<'src, str>>,
    },
    /// Food.
    Food {
        /// Specific food name.
        name: Option<Cow<'src, str>>,
        /// Specific pack.
        pack: Option<Cow<'src, str>>,
    },
    /// Toy entity.
    Toy(Option<Cow<'src, str>>),
    /// Game pack.
    Pack(Option<Cow<'src, str>>),
    /// Effect ability.
    Ability(Option<Cow<'src, str>>),
    /// Food perk.
    Perk(Option<i32>),
    /// Ailment.
    Ailment(Option<i32>),
    /// Spaces.
    Space(Option<i32>),
    /// Phases of battle.
    Battle(Option<i32>),
    /// Turns.
    Turn(Option<i32>),
    /// Attack.
    Attack(Option<i32>),
    /// Attack damage.
    Damage(Option<i32>),
    /// Health.
    Health(Option<i32>),
    /// Gold.
    Gold(Option<i32>),
    /// Trumpets.
    Trumpet(Option<i32>),
    /// Level of item/pet.
    Level(Option<i32>),
    /// Tier of item/pet.
    Tier(Option<i32>),
    /// Number of uses.
    Uses(Option<i32>),
    /// Experience.
    Experience(Option<i32>),

    /// Attack percent.
    AttackPercent(Option<f32>),
    /// Health percent.
    HealthPercent(Option<f32>),
    /// Damage percent.
    DamagePercent(Option<f32>),
    /// Gold percent.
    GoldPercent(Option<f32>),
    /// Trumpet percent.
    TrumpetPercent(Option<f32>),
}

impl<'src> EntityType<'src> {
    /// Value of inner item, if any.
    /// * [`f32`] are coerced to [`i32`] which in SAP values shouldn't be an issue.
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
            EntityType::Pet { .. }
            | EntityType::Food { .. }
            | EntityType::Toy(_)
            | EntityType::Pack(_)
            | EntityType::Ability { .. } => None,
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
            EntityType::Pet { .. }
            | EntityType::Food { .. }
            | EntityType::Toy(_)
            | EntityType::Pack(_)
            | EntityType::Ability { .. } => {}
        }

        Ok(self)
    }
}

impl<'src> FromStr for EntityType<'src> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "pet" | "pets" => EntityType::Pet {
                name: None,
                attr: None,
                pack: None,
            },
            "food" | "foods" => EntityType::Food {
                name: None,
                pack: None,
            },
            "toy" | "toys" => EntityType::Toy(None),
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
            "ability" => EntityType::Ability(None),
            "pack" => EntityType::Pack(None),
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
