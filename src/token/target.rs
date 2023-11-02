//! SAP effect targets.

use std::str::FromStr;

use anyhow::bail;

/// SAP target types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetType {
    /// Friendly pets.
    #[default]
    Friend,
    /// Enemy pets.
    Enemy,
    /// Shop items.
    Shop,
}

impl FromStr for TargetType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "enemy" | "enemies" | "opponent" => TargetType::Enemy,
            "friend" | "friends" | "friendly" => TargetType::Friend,
            "shop" => TargetType::Shop,
            _ => bail!("Not a valid numeric type."),
        })
    }
}
