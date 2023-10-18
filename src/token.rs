use std::fmt::Display;

use anyhow::bail;

use crate::ScannerState;

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'src> {
    /// Type of token.
    pub ttype: TokenType,
    /// Text of token.
    pub text: &'src str,
    /// Token source metadata.
    pub metadata: ScannerState,
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?}) ({})", self.metadata, self.ttype, self.text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // Quantities
    Number,
    Multiplier,
    Percent,
    Plus,
    Minus,

    // Pet Attributes
    AttackAttribute,
    HealthAttribute,
    Attack,
    Damage,
    Health,
    Gold,
    Level,
    Tier,
    Uses,
    Experience,

    // Entity
    Trumpet,
    Strawberry,

    /// End of statement
    End,

    // Positions
    This,
    Adjacent,
    All,
    Any,
    LeftMost,
    RightMost,
    DirectlyBack,
    Whoever,

    // Targets
    Pet,
    Food,
    Perks,
    Enemy,
    Friendly,
    Shop,

    // Conditions/Logic
    If,
    Next,
    Equal,
    UpTo,

    // Actions
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
}

impl TryFrom<Option<&str>> for TokenType {
    type Error = anyhow::Error;

    fn try_from(value: Option<&str>) -> Result<TokenType, Self::Error> {
        // Case-sensitive.

        Ok(match value {
            Some("attack") => TokenType::Attack,
            Some("health") => TokenType::Health,
            Some("Trumpet") | Some("Trumpets") => TokenType::Trumpet,
            Some("damage") => TokenType::Damage,
            Some("gold") => TokenType::Gold,
            Some("times") => TokenType::Multiplier,
            Some(_) => bail!("Invalid token. {value:?}"),
            None => bail!("No value."),
        })
    }
}
