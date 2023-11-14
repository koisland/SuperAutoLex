//! SAP action tokens.

use std::str::FromStr;

use anyhow::bail;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ActionType {
    Choose,
    Deal,
    Gain,
    Give,
    Push,
    Remove,
    Set,
    Spend,
    Stock,
    Summon,
    Swap,
    Break,
    Copy,
    Make,
    Increase,
    Resummon,
    Steal,
    Activate,
    Discount,
    Knock,
    Reduce,
    Swallow,
    Take,
    Transform,
    Replace,
    Shuffle,
    Freeze,
    Unfreeze,

    // Non-Effect
    Attack,
    Eat,
    Buy,
    Sell,
    Upgrade,
    Hurt,
    Faint,
}

impl ActionType {
    /// Check if shop related.
    pub(crate) fn is_shop_related(&self) -> bool {
        matches!(
            self,
            Self::Spend
                | Self::Stock
                | Self::Discount
                | Self::Freeze
                | Self::Unfreeze
                | Self::Eat
                | Self::Buy
                | Self::Sell
                | Self::Upgrade
        )
    }
}

impl FromStr for ActionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "choose" => ActionType::Choose,
            "deal" => ActionType::Deal,
            "gain" | "gained" => ActionType::Gain,
            "give" => ActionType::Give,
            "push" | "pushed" => ActionType::Push,
            "remove" => ActionType::Remove,
            "set" => ActionType::Set,
            "spend" => ActionType::Spend,
            "stock" => ActionType::Stock,
            "summon" | "summoned" => ActionType::Summon,
            "swap" => ActionType::Swap,
            "break" | "broke" => ActionType::Break,
            "copy" => ActionType::Copy,
            "make" => ActionType::Make,
            "increase" => ActionType::Increase,
            "resummon" => ActionType::Resummon,
            "steal" => ActionType::Steal,
            "activate" => ActionType::Activate,
            "discount" => ActionType::Discount,
            "knock" | "knock-out" | "knocked" => ActionType::Knock,
            "reduce" => ActionType::Reduce,
            "swallow" => ActionType::Swallow,
            "take" => ActionType::Take,
            "transform" => ActionType::Transform,
            "replace" => ActionType::Replace,
            "shuffle" => ActionType::Shuffle,
            "freeze" => ActionType::Freeze,
            "unfreeze" => ActionType::Unfreeze,
            "attack" | "attacks" => ActionType::Attack,
            "eat" | "eats" => ActionType::Eat,
            "buy" | "bought " => ActionType::Buy,
            "upgrade" => ActionType::Upgrade,
            "hurt" => ActionType::Hurt,
            "sell" | "sold" => ActionType::Sell,
            "faint" | "faints" | "fainting" => ActionType::Faint,
            _ => bail!("Unknown action. {s}"),
        })
    }
}
