use std::str::FromStr;

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetType {
    #[default]
    Friend,
    Enemy,
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
