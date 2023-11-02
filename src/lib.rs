//! # SAPLex
//! Lexer for Super Auto Pets effects.
//!
//! Partially based on https://craftinginterpreters.com

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

/// SAP effect
pub mod effect;
/// SAP text scanner state.
pub mod scanner;
/// SAP token.
pub mod token;
/// SAP text tokenizer implementation.
pub mod tokenize;
/// SAP effect trigger.
pub mod trigger;

pub use token::{types::TokenType, Token};
pub use tokenize::SAPText;
