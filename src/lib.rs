// https://craftinginterpreters.com/scanning.html

pub mod effect;
pub mod scanner;
pub mod token;
pub mod trigger;

use anyhow::{bail, Context};
use scanner::Scanner;
use token::{
    attribute::EntityType, logic::LogicType, numeric::NumericType, types::TokenType, ParseNumber,
    SAPTokens, Token,
};

#[deny(missing_docs)]
#[deny(clippy::missing_docs_in_private_items)]

/// Check if ascii digit char.
fn is_digit(chr: Option<char>) -> Option<char> {
    chr.filter(|chr| chr.is_ascii_digit())
}

/// Check if is alphabet char or apostrophe.
fn is_alpha(chr: Option<char>) -> Option<char> {
    chr.filter(|chr| chr.is_alphabetic() || *chr == '\'')
}

#[derive(Default)]
pub struct SAPText<'src> {
    /// Raw text.
    pub effect: &'src str,
    /// Lower-case text.
    lowercase_effect: String,
}

impl<'src> SAPText<'src> {
    /// Create SAP effect.
    pub fn new(effect: &'src str) -> SAPText<'src> {
        // Store a lowercase version of effect for case-insensitive token matching.
        SAPText {
            effect,
            lowercase_effect: effect.to_ascii_lowercase(),
        }
    }

    pub fn tokenize(&'src self) -> anyhow::Result<SAPTokens<'src>> {
        let mut tokens = vec![];
        let mut state = Scanner::default();

        loop {
            state.set_start_to_current();
            if self.scan_token(&mut state, &mut tokens)?.is_none() {
                break;
            };
        }

        // End of statement.
        tokens.push(Token {
            ttype: TokenType::End,
            text: "",
            metadata: state,
        });
        Ok(SAPTokens(tokens))
    }

    fn scan_token(
        &'src self,
        state: &mut Scanner,
        tokens: &mut Vec<Token<'src>>,
    ) -> anyhow::Result<Option<()>> {
        // Reached end.
        let Some(c) = self.advance(state) else {
            return Ok(None);
        };

        match c {
            'A'..='Z' | 'a'..='z' => {
                self.scan_word_token(state, tokens)?;
            }
            '+' | '-' => {
                self.scan_sign_token(state, tokens)?;
            }
            '\n' => {
                state.line += 1;
            }
            // Skip punctuation.
            '.' | ',' | ' ' | '\t' | '/' => {}
            // Scan digits.
            '0'..='9' => {
                self.scan_numeric_token(state, tokens)?;
            }
            _ => {
                bail!("{state}. Invalid character ({c})")
            }
        }

        // More to do.
        Ok(Some(()))
    }

    fn scan_word_token(
        &'src self,
        state: &mut Scanner,
        tokens: &mut Vec<Token<'src>>,
    ) -> anyhow::Result<()> {
        // First word will be capitalized.
        let prev_chr = state.start.checked_sub(1).and_then(|idx| self.peek(idx));
        let is_itemname = self
            .peek(state.start)
            .filter(|chr| chr.is_ascii_uppercase() && prev_chr.is_some())
            .is_some();

        while self.advance_by_cond(state, is_alpha).is_some() {}

        let next_chr = self.peek(state.current);

        match (next_chr, is_itemname) {
            // Multi-word item name.
            // ex. Loyal Chinchilla
            // ex. Bus with Chili.
            // ex. Fortune Cookie Perk
            // ex. If item is Apple, ...
            (Some(' ') | Some('.') | Some(','), true) => {
                loop {
                    state.move_cursor(true, 1).set_start_to_current();

                    let prev_curr = state.current;
                    // Consume word. Stop on non-alphabetic char.
                    while self.advance_by_cond(state, is_alpha).is_some() {}

                    // Check if perk or capitalized.
                    let prev_word = self.effect.get(state.start..prev_curr);
                    let is_prev_word_uppercase = prev_word
                        .map(|word| {
                            word.chars()
                                .next()
                                .map_or(false, |chr| chr.is_ascii_uppercase())
                        })
                        .unwrap_or(false);
                    let is_prev_word_food_related = prev_word == Some("perk")
                        || prev_word == Some("Perk")
                        || tokens.last().map(|token| &token.ttype)
                            == Some(&TokenType::Logic(LogicType::With));

                    let ttype = if is_prev_word_food_related {
                        TokenType::Entity(EntityType::Food(None))
                    } else if is_prev_word_uppercase {
                        TokenType::Entity(EntityType::Pet(None))
                    } else {
                        // Not an item we scanned.
                        // Convert to token (if possible), add, and break from loop.
                        if let Some(Ok(ttype)) = prev_word.map(|word| TokenType::parse(word, None))
                        {
                            tokens.push(self.build_token(state, ttype)?);
                        }
                        break;
                    };

                    tokens.push(self.build_token(state, ttype)?);
                }
            }
            // ex. Loyal-Lizard
            (Some(_), true) => {
                bail!("{state} Unknown item name word delimiter. {next_chr:?}")
            }
            // Otherwise, just create current token.
            // ex. attack
            (Some(' '), false) => {
                let word = self.get_text(state, true)?;
                let ttype = TokenType::parse(word, None);

                let prev_curr = state.current;
                while self.advance_by_cond(state, is_digit).is_some() {}
                let digit_str = self
                    .lowercase_effect
                    .get(prev_curr..state.current)
                    .context("Invalid indices.")?;

                match ttype {
                    // If is entity type token, try to add next digit.
                    Ok(TokenType::Entity(mut entity_type)) => {
                        let _ = entity_type.parse_num_str(digit_str);
                        tokens.push(self.build_token(state, TokenType::Entity(entity_type))?);
                    }
                    // Otherwise, add new token.
                    Ok(ttype) => {
                        tokens.push(self.build_token(state, ttype)?);
                    }
                    // Invalid token type. Ignore.
                    _ => {}
                }
            }
            (Some(_), false) | (None, _) => {
                let word = self.get_text(state, true)?;
                if let Ok(ttype) = TokenType::parse(word, None) {
                    tokens.push(self.build_token(state, ttype)?);
                }
            }
        }

        Ok(())
    }

    fn scan_sign_token(
        &'src self,
        state: &mut Scanner,
        tokens: &mut Vec<Token<'src>>,
    ) -> anyhow::Result<()> {
        state.set_start_to_current();

        // Keep reading until not a digit.
        // state.current now points to char after digits.
        while self.advance_by_cond(state, is_digit).is_some() {}

        // Include sign in value.
        let mut num_literal_state = state.clone();
        num_literal_state.move_cursor(false, -1);

        let next_chr = self.peek(state.current);
        match next_chr {
            // Raw attribute number.
            // ex. +1 attack
            Some(' ') => {
                let token = self.consume_while_cond(state, Some(num_literal_state), 1, is_alpha)?;
                tokens.push(token)
            }
            // Percentage multiplier.
            // ex. +100% trumpets
            Some('%') => {
                let mut token =
                    self.consume_while_cond(state, Some(num_literal_state), 2, is_alpha)?;
                if let TokenType::Entity(ref mut attr_type) = token.ttype {
                    *attr_type = attr_type.into_percent_variant()?;
                }
                tokens.push(token)
            }
            Some(_) => {
                bail!("{state} Non-whitespace {next_chr:?} after digit.");
            }
            None => todo!(),
        }

        Ok(())
    }

    fn scan_numeric_token(
        &'src self,
        state: &mut Scanner,
        tokens: &mut Vec<Token<'src>>,
    ) -> anyhow::Result<()> {
        // Keep going if digit. ex. '12/12'
        while self.advance_by_cond(state, is_digit).is_some() {}

        let num_literal_state = state.clone();

        match self.peek(state.current) {
            // ex. 12/12
            Some('/') => {
                tokens.push(self.build_token(
                    state,
                    TokenType::Entity(EntityType::Attack(Some(
                        self.get_text(&num_literal_state, false)?.parse()?,
                    ))),
                )?);

                // Registers as numeric since no attribute text.
                // Change so is correctly labeled health.
                let mut health_token = self.consume_while_cond(state, None, 1, is_digit)?;
                health_token.ttype =
                    TokenType::Entity(EntityType::Health(Some(health_token.text.parse()?)));
                tokens.push(health_token)
            }
            // ex. 1 attack
            // ex. 1-gold
            Some(' ') | Some('-') => {
                let num_literal_token = self.build_token(
                    &num_literal_state,
                    TokenType::Numeric(NumericType::Number(Some(
                        self.get_text(&num_literal_state, false)?.parse()?,
                    ))),
                )?;
                let next_token =
                    self.consume_while_cond(state, Some(num_literal_state), 1, is_alpha)?;
                // Only add token if next token related to entities.
                match next_token.ttype {
                    TokenType::Numeric(_) | TokenType::Entity(_) => tokens.push(next_token),
                    _ => tokens.push(num_literal_token),
                }
            }
            // Ignore everything else and just add number.
            Some(_) | None => tokens.push(self.build_token(
                state,
                TokenType::Numeric(NumericType::Number(Some(
                    self.get_text(&num_literal_state, false)?.parse()?,
                ))),
            )?),
        }

        Ok(())
    }

    /// Peek at index character without advancing `SAPText`.
    /// * Note: This will use the raw effect source and not the lowercase version.
    fn peek(&self, idx: usize) -> Option<char> {
        self.effect
            .as_bytes()
            .get(idx)
            .filter(|byte| byte.is_ascii())
            .map(|byte| *byte as char)
    }

    /// Consume characters in [`SAPText`] [`Scanner`] building a [`Token`] while the provided condition is valid.
    ///
    /// ### Params
    /// * `state`
    ///     * [`Scanner`] to be modified.
    /// * `literal_state`
    ///     * Optional state to be used to construct a [`Token`]'s literal value.
    ///     * Also providing will in
    /// * `cur_adj`
    ///     * Cursor adjustment for `state`'s [`Scanner::current`].
    /// * `cond`
    ///     * Closure that checks if current character is valid.
    ///
    /// ### Returns
    /// * [`Token`]
    fn consume_while_cond(
        &'src self,
        state: &mut Scanner,
        literal_state: Option<Scanner>,
        cur_adj: isize,
        cond: impl Fn(Option<char>) -> Option<char>,
    ) -> anyhow::Result<Token<'src>> {
        state.move_cursor(true, cur_adj).set_start_to_current();

        // Move cursor while condition is met.
        while self.advance_by_cond(state, &cond).is_some() {}

        let word = self.get_text(state, true)?;
        if let Some(mut updated_literal_state) = literal_state {
            let literal_value = self.get_text(&updated_literal_state, false)?;
            // Use literal state updated so Token text includes both literal value and attribute token.
            updated_literal_state.current = state.current;
            self.build_token(
                &updated_literal_state,
                TokenType::parse(word, Some(literal_value))?,
            )
        } else {
            self.build_token(state, TokenType::parse(word, None)?)
        }
    }

    fn get_text(&'src self, state: &Scanner, lowercase: bool) -> anyhow::Result<&'src str> {
        let source = if lowercase {
            &self.lowercase_effect
        } else {
            self.effect
        };
        source.get(state.start..state.current).with_context(|| {
            format!(
                "Invalid start {} and current {} indices in source.",
                state.start, state.current
            )
        })
    }

    fn build_token(&self, state: &Scanner, ttype: TokenType) -> anyhow::Result<Token> {
        Ok(Token {
            ttype,
            text: self.get_text(state, false)?,
            metadata: state.clone(),
        })
    }

    /// Advances [`Scanner`]
    fn advance(&self, state: &mut Scanner) -> Option<char> {
        if let Some(char) = self.peek(state.current) {
            state.current += 1;
            Some(char)
        } else {
            None
        }
    }

    /// Conditional [`SAPText::advance`].
    ///
    /// ### Params
    /// * `state`
    ///     * Current state of [`SAPText`].
    /// * `pass_cond`
    ///     * Closure taking the next character and return an optional char.
    ///     * Passes if [`Option::Some`] and increments `state.current` char position.
    ///
    /// ### Returns
    /// * Next character or [`Option::None`].
    fn advance_by_cond(
        &self,
        state: &mut Scanner,
        pass_cond: impl Fn(Option<char>) -> Option<char>,
    ) -> Option<char> {
        if let Some(chr) = pass_cond(self.peek(state.current)) {
            state.current += 1;
            Some(chr)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::token::actions::ActionType;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_tokenize_sign_numeric_attr() {
        let effect = SAPText::new("Gain +3 attack and +2 health.");
        let tokens = effect.tokenize().unwrap();

        assert_eq!(
            *tokens,
            vec![
                Token {
                    ttype: TokenType::Action(ActionType::Gain),
                    text: "Gain",
                    metadata: Scanner {
                        start: 0,
                        current: 4,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Entity(EntityType::Attack(Some(3))),
                    text: "+3 attack",
                    metadata: Scanner {
                        start: 5,
                        current: 14,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Logic(LogicType::And),
                    text: "and",
                    metadata: Scanner {
                        start: 15,
                        current: 18,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Entity(EntityType::Health(Some(2))),
                    text: "+2 health",
                    metadata: Scanner {
                        start: 19,
                        current: 28,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::End,
                    text: "",
                    metadata: Scanner {
                        start: 29,
                        current: 29,
                        line: 1
                    }
                }
            ]
        )
    }
    #[test]
    fn test_tokenize_sign_numeric_perc_attr() {
        let valid_attr_num = SAPText::new("+100% health and +120% attack");
        let tokens = valid_attr_num.tokenize().unwrap();

        assert_eq!(
            *tokens,
            vec![
                Token {
                    ttype: TokenType::Entity(EntityType::HealthPercent(Some(100.0))),
                    text: "+100% health",
                    metadata: Scanner {
                        start: 0,
                        current: 12,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Logic(LogicType::And),
                    text: "and",
                    metadata: Scanner {
                        start: 13,
                        current: 16,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Entity(EntityType::AttackPercent(Some(120.0))),
                    text: "+120% attack",
                    metadata: Scanner {
                        start: 17,
                        current: 29,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::End,
                    text: "",
                    metadata: Scanner {
                        start: 29,
                        current: 29,
                        line: 1
                    }
                }
            ]
        )
    }

    #[test]
    fn test_tokenize_numeric_attr() {
        let valid_attr_num = SAPText::new("1-gold");
        let tokens = valid_attr_num.tokenize().unwrap();

        assert_eq!(
            *tokens,
            vec![
                Token {
                    ttype: TokenType::Entity(EntityType::Gold(Some(1))),
                    text: "1-gold",
                    metadata: Scanner {
                        start: 0,
                        current: 6,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::End,
                    text: "",
                    metadata: Scanner {
                        start: 6,
                        current: 6,
                        line: 1
                    }
                }
            ]
        )
    }

    #[test]
    fn test_tokenize_numeric_summon_stats() {
        let valid_summon_stats = SAPText::new("12/13");
        let invalid_summon_stats_health_missing = SAPText::new("12/");
        let invalid_summon_stats_health_nondigit = SAPText::new("12/a");

        assert!(invalid_summon_stats_health_missing.tokenize().is_err());
        assert!(invalid_summon_stats_health_nondigit.tokenize().is_err());

        let tokens = valid_summon_stats.tokenize().unwrap();
        assert_eq!(
            *tokens,
            vec![
                Token {
                    ttype: TokenType::Entity(EntityType::Attack(Some(12))),
                    text: "12",
                    metadata: Scanner {
                        start: 0,
                        current: 2,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Entity(EntityType::Health(Some(13))),
                    text: "13",
                    metadata: Scanner {
                        start: 3,
                        current: 5,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::End,
                    text: "",
                    metadata: Scanner {
                        start: 5,
                        current: 5,
                        line: 1
                    }
                }
            ]
        )
    }
}
