//! SAP item positions inside/outside of battle.
use std::str::FromStr;

use anyhow::bail;

/// SAP item positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionType {
    /// This pet.
    OnSelf,
    /// Not this pet.
    NonSelf,
    /// Ahead of pet.
    Ahead,
    /// Behind pet.
    Behind,
    /// Nearest pet relative to current pet.
    Nearest,
    /// Pets directly one space ahead and behind of current pet.
    Adjacent,
    /// All items.
    All,
    /// Any item.
    Any,
    /// Highest item of some attribute.
    Highest,
    /// Lowest item of some attribute.
    Lowest,
    /// Left-most item. This is the last element in the set.
    LeftMost,
    /// Right-most item. This is the first element in the set.
    RightMost,
    /// Item causing this effect to trigger.
    Trigger,
    /// Lowest health pet.
    Illest,
    /// Highest health pet.
    Healthiest,
    /// Highest attack pet.
    Strongest,
    /// Lowest attack pet.
    Weakest,
    /// Pet directly opposite of this pet.
    Opposite,
}

impl FromStr for PositionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "this" | "itself" => PositionType::OnSelf,
            "other" | "nonself" => PositionType::NonSelf,
            "ahead" | "forward" => PositionType::Ahead,
            "behind" => PositionType::Behind,
            "adjacent" => PositionType::Adjacent,
            "nearest" => PositionType::Nearest,
            "all" => PositionType::All,
            "random" | "any" => PositionType::Any,
            "highest" => PositionType::Highest,
            "lowest" => PositionType::Lowest,
            "left-most" => PositionType::LeftMost,
            "right-most" | "front" => PositionType::RightMost,
            "directly back" | "whoever" | "it" | "its" => PositionType::Trigger,
            "most healthy" => PositionType::Healthiest,
            "strongest" => PositionType::Strongest,
            "weakest" => PositionType::Weakest,
            "opposite" => PositionType::Opposite,
            _ => bail!("{s} not a valid PositionType"),
        })
    }
}
