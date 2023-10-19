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
    DirectlyBack,
    Whoever,
    That,
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
            "directly back" => PositionType::DirectlyBack,
            "whoever" => PositionType::Whoever,
            "that" => PositionType::That,
            _ => bail!("{s} not a valid PositionType"),
        })
    }
}
