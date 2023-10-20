use std::str::FromStr;

use anyhow::bail;

use super::{
    actions::ActionType, attribute::AttributeType, logic::LogicType, numeric::NumericType,
    position::PositionType, target::TargetType, ParseNumber,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Numeric(NumericType),
    Attribute(AttributeType),
    End,
    Position(PositionType),
    Target(TargetType),
    Logic(LogicType),
    Action(ActionType),
}

impl TokenType {
    /// Parse text into a [`TokenType`].
    ///
    /// ### Params
    /// * `ttype_str`
    ///     * Token type word to be parsed.
    /// * `literal_str`
    ///     * Optional literal value for [`TokenType`]
    ///     * ex. `1 attack`
    ///         * `1` is the literal value represented by `"1"`.
    ///
    /// ### Returns
    /// * Parsed [`TokenType`]
    /// * Errors if cannot convert value to a [`TokenType`] variant.
    pub fn parse(ttype_str: &str, literal_str: Option<&str>) -> anyhow::Result<TokenType> {
        Ok(
            if let Ok(mut attr_type) = AttributeType::from_str(ttype_str) {
                // Add number to attribute if provided.
                if let Some(literal_str) = literal_str {
                    attr_type.parse_num_str(literal_str)?;
                };
                TokenType::Attribute(attr_type)
            } else if let Ok(pos_type) = PositionType::from_str(ttype_str) {
                TokenType::Position(pos_type)
            } else if let Ok(mut num_type) = NumericType::from_str(ttype_str) {
                // Add number to numeric type if provided.
                if let Some(literal_str) = literal_str {
                    num_type.parse_num_str(literal_str)?;
                };
                TokenType::Numeric(num_type)
            } else if let Ok(action_type) = ActionType::from_str(ttype_str) {
                TokenType::Action(action_type)
            } else if let Ok(target_type) = TargetType::from_str(ttype_str) {
                TokenType::Target(target_type)
            } else if let Ok(logic_type) = LogicType::from_str(ttype_str) {
                TokenType::Logic(logic_type)
            } else if let Ok(num) = ttype_str.parse::<i32>() {
                TokenType::Numeric(NumericType::Number(Some(num)))
            } else {
                bail!("Not a valid token type. {ttype_str}")
            },
        )
    }
}
