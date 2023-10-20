use std::str::FromStr;

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionType {
    // Positions
    OnSelf,
    Ahead,
    Behind,
    Adjacent,
    All,
    Any,
    Highest,
    Lowest,
    LeftMost,
    RightMost,
    Trigger,
}

impl FromStr for PositionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "this" => PositionType::OnSelf,
            "ahead" => PositionType::Ahead,
            "behind" => PositionType::Behind,
            "adjacent" => PositionType::Adjacent,
            "all" => PositionType::All,
            "random" | "any" => PositionType::Any,
            "highest" => PositionType::Highest,
            "lowest" => PositionType::Lowest,
            "left-most" => PositionType::LeftMost,
            "right-most" => PositionType::RightMost,
            "directly back" | "whoever" | "it" => PositionType::Trigger,
            _ => bail!("{s} not a valid PositionType"),
        })
    }
}
