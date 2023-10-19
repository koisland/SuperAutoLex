use std::str::FromStr;

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    Pet,
    Food,
    Perk,
    Friend,
    Enemy,
    Shop,
}

impl FromStr for TargetType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "pet" | "pets" => TargetType::Pet,
            "food" | "foods" => TargetType::Food,
            "perk" | "perks" => TargetType::Perk,
            "enemy" | "enemies" => TargetType::Enemy,
            "friend" | "friends" | "friendly" => TargetType::Friend,
            "shop" => TargetType::Shop,
            _ => bail!("Not a valid numeric type."),
        })
    }
}
