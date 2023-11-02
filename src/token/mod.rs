//! Token Types

use std::{fmt::Display, ops::Deref};

use crate::scanner::Scanner;

pub mod actions;
pub mod attribute;
pub mod logic;
pub mod numeric;
pub mod position;
pub mod target;
pub mod types;

use self::types::TokenType;

pub use self::{
    actions::ActionType, attribute::EntityType, logic::LogicType, numeric::NumericType,
    position::PositionType, target::TargetType,
};

/// A SAP text token.
#[derive(Debug, PartialEq)]
pub struct Token<'src> {
    /// Type of token.
    pub ttype: TokenType<'src>,
    /// Text of token.
    pub text: &'src str,
    /// Token source metadata.
    pub metadata: Scanner,
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?}) ({})", self.metadata, self.ttype, self.text)
    }
}

/// Parse number.
pub(crate) trait ParseNumber {
    /// Parsed numeric string and modify [`Self`] with it.
    fn parse_num_str(&mut self, num_str: &str) -> anyhow::Result<&mut Self>
    where
        Self: Sized;
}

/// Wrapper for [`Vec<Token>`].
#[derive(Debug, PartialEq)]
pub struct SAPTokens<'src>(pub Vec<Token<'src>>);

impl<'src> Deref for SAPTokens<'src> {
    type Target = [Token<'src>];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}
