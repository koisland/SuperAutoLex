use std::str::FromStr;

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionType {
    // Positions
    OnSelf,
    NonSelf,
    Ahead,
    Behind,
    Nearest,
    Adjacent,
    All,
    Any,
    Highest,
    Lowest,
    LeftMost,
    RightMost,
    Trigger,
    Illest,
    Healthiest,
    Strongest,
    Weakest,
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
