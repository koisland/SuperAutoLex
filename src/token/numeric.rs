use std::str::FromStr;

use anyhow::bail;

use super::ParseNumber;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericType {
    Number(Option<i32>),
    Multiplier(Option<i32>),
    Percent(Option<f32>),
    Sum,
    Plus,
    Minus,
}

/// Coerces solely string numeric type.
/// ex. one, double, etc.
impl FromStr for NumericType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "times" => NumericType::Multiplier(None),
            "one" => NumericType::Number(Some(1)),
            "two" => NumericType::Number(Some(2)),
            "three" => NumericType::Number(Some(3)),
            "double" => NumericType::Multiplier(Some(2)),
            "triple" => NumericType::Multiplier(Some(3)),
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
            NumericType::Sum => todo!(),
            NumericType::Plus => todo!(),
            NumericType::Minus => todo!(),
        }
        Ok(self)
    }
}
