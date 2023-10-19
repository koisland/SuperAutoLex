use std::str::FromStr;

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicType {
    If,
    Or,
    Next,
    Equal,
    UpTo,
    With,
}

impl FromStr for LogicType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "if" => LogicType::If,
            "or" => LogicType::Or,
            "next" => LogicType::Next,
            "equal" => LogicType::Equal,
            "up to" => LogicType::UpTo,
            "with" => LogicType::With,
            _ => bail!("Not a valid LogicType. {s}"),
        })
    }
}
