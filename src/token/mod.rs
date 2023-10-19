use std::fmt::Display;

use crate::Scanner;

use self::types::TokenType;

pub mod actions;
pub mod attribute;
pub mod logic;
pub mod numeric;
pub mod position;
pub mod target;
pub mod trigger;
pub mod types;

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'src> {
    /// Type of token.
    pub ttype: TokenType,
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
