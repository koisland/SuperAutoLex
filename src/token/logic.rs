use std::str::FromStr;

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicType {
    If,
    // If next lexeme is higher or lower switch to GreaterEqual or LessEqual. Otherwise, do nothing.
    Or,
    Next,
    LessEqual,
    Equal,
    GreaterEqual,
    UpTo,
    With,
    ForEach
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
            "for each" => LogicType::ForEach,
            _ => bail!("Not a valid LogicType. {s}"),
        })
    }
}
