use std::str::FromStr;

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicType {
    If,
    // If next lexeme is higher or lower switch to GreaterEqual or LessEqual. Otherwise, do nothing.
    Or,
    And,
    Start,
    End,
    Before,
    After,
    Then,
    Until,
    With,
    Works,
    Have,
    For,
    Each,
    ForEach,
    Except,
    // For targets and trumpet effects.
    To,
    // In or outside battle.
    In,
    Outside,
}

impl FromStr for LogicType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "if" => LogicType::If,
            "and" => LogicType::And,
            "then" => LogicType::Then,
            "until" => LogicType::Until,
            "or" => LogicType::Or,
            "start" => LogicType::Start,
            "end" => LogicType::End,
            "with" => LogicType::With,
            "for" => LogicType::For,
            "has" | "have" => LogicType::Have,
            "each" => LogicType::Each,
            "for each" => LogicType::ForEach,
            "before" => LogicType::Before,
            "after" => LogicType::After,
            "works" => LogicType::Works,
            "except" => LogicType::Except,
            "in" => LogicType::In,
            "to" => LogicType::To,
            "outside" => LogicType::Outside,
            _ => bail!("Not a valid LogicType. {s}"),
        })
    }
}
