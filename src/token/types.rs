use std::str::FromStr;

use anyhow::bail;

use super::{
    actions::ActionType, attribute::AttributeType, logic::LogicType, numeric::NumericType,
    position::PositionType, target::TargetType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Numeric(NumericType),
    Attribute(AttributeType),
    End,
    Position(PositionType),
    Target(TargetType),
    Logic(LogicType),
    Action(ActionType),
}

impl FromStr for TokenType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Ok(attr_type) = AttributeType::from_str(s) {
            TokenType::Attribute(attr_type)
        } else if let Ok(pos_type) = PositionType::from_str(s) {
            TokenType::Position(pos_type)
        } else if let Ok(num_type) = NumericType::from_str(s) {
            TokenType::Numeric(num_type)
        } else if let Ok(action_type) = ActionType::from_str(s) {
            TokenType::Action(action_type)
        } else if let Ok(target_type) = TargetType::from_str(s) {
            TokenType::Target(target_type)
        } else if let Ok(logic_type) = LogicType::from_str(s) {
            TokenType::Logic(logic_type)
        } else {
            bail!("Not a valid token type.")
        })
    }
}
