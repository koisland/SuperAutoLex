use std::str::FromStr;

use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericType {
    Number,
    Multiplier,
    Percent,
    Plus,
    Minus,
}

/// Coerces solely string numeric type.
/// ex. one, double, etc.
impl FromStr for NumericType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "times" => NumericType::Multiplier,
            "one" | "two" | "three" => NumericType::Number,
            "double" | "triple" => NumericType::Multiplier,
            _ => bail!("Not a valid numeric type."),
        })
    }
}
