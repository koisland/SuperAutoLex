use std::str::FromStr;

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Friend,
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
    Unfreeze,

    // Non-Effect
    Attack,
    Eat,
    Buy,
    Upgrade,
    Hurt,
    Sold,
}

impl FromStr for ActionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "choose" => ActionType::Choose,
            "deal" => ActionType::Deal,
            "gain" => ActionType::Gain,
            "give" => ActionType::Give,
            "push" => ActionType::Push,
            "remove" => ActionType::Remove,
            "set" => ActionType::Set,
            "spend" => ActionType::Spend,
            "stock" => ActionType::Stock,
            "summon" => ActionType::Summon,
            "swap" => ActionType::Swap,
            "break" | "broke" => ActionType::Break,
            "copy" => ActionType::Copy,
            "make" => ActionType::Make,
            "friend" => ActionType::Friend,
            "increase" => ActionType::Increase,
            "resummon" => ActionType::Resummon,
            "steal" => ActionType::Steal,
            "activate" => ActionType::Activate,
            "discount" => ActionType::Discount,
            "knock" | "knock-out" => ActionType::Knock,
            "reduce" => ActionType::Reduce,
            "swallow" => ActionType::Swallow,
            "take" => ActionType::Take,
            "transform" => ActionType::Transform,
            "replace" => ActionType::Replace,
            "shuffle" => ActionType::Shuffle,
            "unfreeze" => ActionType::Unfreeze,
            "attack" | "attacks" => ActionType::Attack,
            "eat" | "eats" => ActionType::Eat,
            "buy" => ActionType::Buy,
            "upgrade" => ActionType::Upgrade,
            "hurt" => ActionType::Hurt,
            "sold" => ActionType::Sold,
            _ => bail!("Unknown action. {s}"),
        })
    }
}
