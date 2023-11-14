//! SAP logic.

use std::str::FromStr;

use anyhow::bail;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Logic related tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LogicType {
    /// If a condition.
    /// - ex. `If in battle, ...`
    If,
    /// Subject is.
    Is,
    /// Or some other condition.
    /// - `End of turn or end of battle`
    // If next lexeme is higher or lower switch to GreaterEqual or LessEqual. Otherwise, do nothing.
    Or,
    /// And
    ///
    And,
    /// Start of something.
    /// - ex. `Start of battle`
    Start,
    /// End of soemthing.
    /// - ex. `End turn`
    End,
    /// Before something.
    Before,
    /// After something.
    After,
    /// Indicates second action.
    /// - ex. `If ... then, ...`
    Then,
    /// Until something.
    Until,
    /// With some item.
    /// - ex. `Deer with Chili`
    With,
    /// Number of times an effect works.
    Works,
    /// Condition of having something.
    Have,
    /// Part of [`LogicType::ForEach`]
    For,
    /// Part of [`LogicType::ForEach`]
    Each,
    /// For each of a condition.
    /// - `..., for each Strawberry friend, ... `
    ForEach,
    /// Exceptions to an effect.
    /// - ex. `Except other Tapirs!`
    Except,
    /// For targets and trumpet effects.
    To,
    /// In battle.
    In,
    /// Outside of battle.
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
            "is" => LogicType::Is,
            "has" | "have" => LogicType::Have,
            "each" | "every" => LogicType::Each,
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
