use std::str::FromStr;

use anyhow::bail;

use super::ParseNumber;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericType {
    Number(Option<i32>),
    Multiplier(Option<i32>),
    Percent(Option<f32>),
    Plus,
    Minus,
    LessEqual,
    Equal,
    GreaterEqual,
    Sum,
    Max,
    Min,
}

/// Coerces solely string numeric type.
/// ex. one, double, etc.
impl FromStr for NumericType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "time" | "times" => NumericType::Multiplier(None),
            "one" => NumericType::Number(Some(1)),
            "two" => NumericType::Number(Some(2)),
            "three" => NumericType::Number(Some(3)),
            "four" => NumericType::Number(Some(4)),
            "five" => NumericType::Number(Some(5)),
            "six" => NumericType::Number(Some(6)),
            "seven" => NumericType::Number(Some(7)),
            "double" => NumericType::Multiplier(Some(2)),
            "triple" => NumericType::Multiplier(Some(3)),
            "lower" => NumericType::LessEqual,
            "equal" => NumericType::Equal,
            "greater" => NumericType::GreaterEqual,
            "most" => NumericType::Max,
            "least" => NumericType::Min,
            _ => bail!("Not a valid numeric type."),
        })
    }
}

impl ParseNumber for NumericType {
    fn parse_num_str(&mut self, num_str: &str) -> anyhow::Result<&mut Self>
    where
        Self: Sized,
    {
        match self {
            NumericType::Number(ref mut v) | NumericType::Multiplier(ref mut v) => {
                v.replace(num_str.parse()?);
            }
            NumericType::Percent(ref mut v) => {
                v.replace(num_str.parse()?);
            }
            NumericType::Sum
            | NumericType::Plus
            | NumericType::Minus
            | NumericType::LessEqual
            | NumericType::Equal
            | NumericType::GreaterEqual
            | NumericType::Max
            | NumericType::Min => {}
        }
        Ok(self)
    }
}
