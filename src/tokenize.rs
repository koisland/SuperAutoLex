use std::slice::SliceIndex;

use crate::{
    scanner::Scanner,
    token::{
        attribute::EntityType, logic::LogicType, numeric::NumericType, position::PositionType,
        types::TokenType, ParseNumber, SAPTokens, Token,
    },
};
use anyhow::{bail, Context};

/// Check if ascii digit char.
fn is_digit(chr: Option<char>) -> Option<char> {
    chr.filter(|chr| chr.is_ascii_digit())
}

/// Check if is alphabet char or apostrophe.
fn is_alpha(chr: Option<char>) -> Option<char> {
    chr.filter(|chr| chr.is_alphabetic() || *chr == '\'')
}

/// Super Auto Pets text.
#[derive(Default)]
pub struct SAPText<'src> {
    /// Raw text.
    pub effect: &'src str,
    /// Lower-case text.
    lowercase_effect: String,
}

impl<'src> SAPText<'src> {
    /// Create new SAP text.
    ///
    /// ```
    /// use saplex::SAPText;
    ///
    /// let trigger = SAPText::new("End Turn");
    /// let effect = SAPText::new("Gain +2 attack and +2 health.");
    /// ```
    pub fn new(effect: &'src str) -> SAPText<'src> {
        // Store a lowercase version of effect for case-insensitive token matching.
        SAPText {
            effect,
            lowercase_effect: effect.to_ascii_lowercase(),
        }
    }

    /// Tokenize text.
    /// - Any uppercase text is treated as an itemname unless it is at the start of the text.
    ///     - ex. `Gain Lemon.`
    /// - Most punctuation is ignored.
    ///
    /// ```
    /// use saplex::{
    ///     SAPText, Token, TokenType,
    ///     scanner::Scanner,
    ///     token::{LogicType, EntityType}
    /// };
    ///
    /// let trigger = SAPText::new("End turn");
    /// assert_eq!(
    ///     *trigger::tokenize().unwrap,
    ///     [
    ///         Token {
    ///             ttype: TokenType::Logic(LogicType::End),
    ///             text: "End",
    ///             metadata: Scanner { start: 0, current: 3, line: 1 }
    ///         },
    ///         Token {
    ///             ttype: TokenType::Entity(EntityType::Turn(None)),
    ///             text: "turn",
    ///             metadata: Scanner { start: 4, current: 8, line: 1 }
    ///         },
    ///         Token {
    ///             ttype: TokenType::EndText,
    ///             text: "",
    ///             metadata: Scanner { start: 8, current: 8, line: 1 }
    ///         }
    ///     ]
    /// )
    /// ````
    pub fn tokenize(&'src self) -> anyhow::Result<SAPTokens<'src>> {
        let mut tokens = vec![];
        let mut state = Scanner::default();

        loop {
            state.set_start_to_current();
            if self.scan_token(&mut state, &mut tokens)?.is_none() {
                break;
            };
        }

        // EndText of statement.
        tokens.push(Token {
            ttype: TokenType::EndText,
            text: "",
            metadata: state,
        });
        Ok(SAPTokens(tokens))
    }

    /// Scans a character and if meets some conditions, consumes remaining characters to create zero or more tokens.
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

    /// Replaces next alphabetic token if meets condition.
    fn add_multi_token_by_cond(
        &'src self,
        state: &mut Scanner,
        prev_state: Option<&mut Scanner>,
        prev_ttype: Option<TokenType<'src>>,
        filter_fn: impl FnOnce(&Token<'src>) -> bool,
        replacement_ttype: TokenType<'src>,
        tokens: &mut Vec<Token<'src>>,
    ) -> anyhow::Result<Option<()>> {
        let next_alpha_token: Option<Token<'src>> =
            self.consume_while_cond(state, None, 0, is_alpha);
        if let Some(next_token) = next_alpha_token {
            // Check token meets condition.
            if filter_fn(&next_token) {
                // Use prev state, if provided, to include current token in text.
                let adj_state = if let Some(prev_state) = prev_state {
                    prev_state.current = state.current;
                    prev_state
                } else {
                    state
                };
                tokens.push(self.build_token(adj_state, replacement_ttype)?);

                Ok(Some(()))
            } else {
                // Add prev and next token to avoid losing since state advanced.
                if let (Some(prev_state), Some(prev_ttype)) = (prev_state, prev_ttype) {
                    tokens.push(self.build_token(&prev_state, prev_ttype)?)
                }
                tokens.push(next_token);
                Ok(None)
            }
        } else if let (Some(prev_state), Some(prev_ttype)) = (prev_state, prev_ttype) {
            // Add previous token since no next token
            tokens.push(self.build_token(&prev_state, prev_ttype)?);
            Ok(None)
        } else {
            Ok(None)
        }
    }

    /// Scans any alphabetic token.
    fn scan_word_token(
        &'src self,
        state: &mut Scanner,
        tokens: &mut Vec<Token<'src>>,
    ) -> anyhow::Result<()> {
        // First word will be capitalized.
        let prev_chr = state.start.checked_sub(1).and_then(|idx| self.peek(idx));
        let is_itemname = self
            .peek(state.start)
            // Only item if there's a character before and first char is uppercase.
            .filter(|chr| chr.is_ascii_uppercase() && prev_chr.is_some())
            .is_some();

        while self.advance_by_cond(state, is_alpha).is_some() {}

        let next_chr = self.peek(state.current);

        match (next_chr, is_itemname) {
            // Multi-word item name.
            // ex. Loyal Chinchilla
            // ex. Bus with Chili.
            // ex. Fortune Cookie Perk
            (Some(' '), true) => {
                let start_of_word = state.start;
                let mut entity: Option<EntityType> = None;
                // Certain pet attributes are capitalized.
                // ex. "Faint pet" or "Strawberry friend".
                let mut is_pet_attr = false;

                loop {
                    state.move_cursor(true, 1);

                    let prev_curr = state.current;
                    // Consume word. Stop on non-alphabetic char.
                    while self.advance_by_cond(state, is_alpha).is_some() {}

                    // Check if perk or capitalized.
                    let next_word = self
                        .effect
                        .get(prev_curr..state.current)
                        .filter(|word| !word.is_empty());

                    let is_next_word_uppercase = next_word
                        .map(|word| {
                            word.chars()
                                .next()
                                .map_or(false, |chr| chr.is_ascii_uppercase())
                        })
                        .unwrap_or(false);

                    // Check if pet attr. If true, stop checking.
                    if !is_pet_attr {
                        is_pet_attr = next_word == Some("friend")
                            || next_word == Some("friends")
                            || next_word == Some("pet");
                    }

                    // Perks have suffix word "perk"
                    let is_next_word_food_related = next_word == Some("perk")
                        || next_word == Some("Perk")
                        || tokens.last().map(|token| &token.ttype)
                            == Some(&TokenType::Logic(LogicType::With));

                    // Set entity if meet condition.
                    match (
                        is_next_word_uppercase || is_pet_attr,
                        is_next_word_food_related,
                    ) {
                        (true, true) | (false, true) => {
                            entity.replace(EntityType::Food {
                                number: None,
                                name: None,
                            });
                        }
                        (true, false) => {
                            entity.replace(EntityType::Pet {
                                number: None,
                                name: None,
                                // Assign attribute if any.
                                attr: is_pet_attr.then_some(
                                    self.get_text_slice(state.start..prev_curr - 1, false)?,
                                ),
                            });
                        }
                        // Hit unrelated word.
                        (false, false) => {
                            // Reset position to before next word.
                            state.current = prev_curr - 1;
                            break;
                        }
                    }
                    if !(is_next_word_uppercase || is_next_word_food_related) {
                        break;
                    }
                }
                let word = self.get_text_slice(start_of_word..state.current, false)?;

                let token = match entity {
                    Some(
                        EntityType::Food { ref mut name, .. }
                        | EntityType::Pet { ref mut name, .. },
                    ) => {
                        // Only assign name if not a pet attribute.
                        if !is_pet_attr {
                            name.replace(word);
                        }
                        state.start = start_of_word;
                        // Safe to unwrap as checked some entity.
                        self.build_token(state, TokenType::Entity(entity.unwrap()))?
                    }
                    _ => {
                        // Get lowercase effect for parsing.
                        let lowercase_word =
                            self.get_text_slice(start_of_word..state.current, true)?;
                        // Try to parse word defaulting to assuming is pet name.
                        let ttype = TokenType::parse(lowercase_word, None).unwrap_or(
                            TokenType::Entity(EntityType::Pet {
                                number: None,
                                name: Some(word),
                                attr: None,
                            }),
                        );
                        self.build_token(state, ttype)?
                    }
                };
                tokens.push(token)
            }
            // Non-item name word token.
            // ex. attack
            (Some(' '), false) => {
                let word = self.get_text(state, true)?;
                let ttype = TokenType::parse(word, None);

                // Consume digits ahead to create numeric token, if anys.
                let mut prev_state = state.clone();
                let next_digit_token = self.consume_while_cond(state, None, 1, is_digit);

                match ttype {
                    // If is entity type token, try to add next digit, if any.
                    Ok(TokenType::Entity(mut entity_type)) => {
                        if let Some(digit_token) = &next_digit_token {
                            prev_state.current = state.current;
                            let _ = entity_type.parse_num_str(digit_token.text);
                        }
                        tokens.push(self.build_token(&prev_state, TokenType::Entity(entity_type))?);

                        // Early return to avoid adding numeric token twice.
                        return Ok(());
                    }
                    // ex. "this has"
                    // If next token isn't "have", just add "this".
                    // Otherwise, doesn't include this.
                    Ok(TokenType::Position(PositionType::OnSelf)) => {
                        self.add_multi_token_by_cond(
                            state,
                            Some(&mut prev_state),
                            Some(ttype?),
                            |token| token.ttype == TokenType::Logic(LogicType::Have),
                            TokenType::Logic(LogicType::Have),
                            tokens,
                        )?;
                    }
                    // ex. "for each"
                    // Normal situations should only include "for" with "each"/"every". Ignore others and don't add "for".
                    Ok(TokenType::Logic(LogicType::For)) => {
                        self.add_multi_token_by_cond(
                            state,
                            Some(&mut prev_state),
                            None,
                            |token| token.ttype == TokenType::Logic(LogicType::Each),
                            TokenType::Logic(LogicType::ForEach),
                            tokens,
                        )?;
                    }
                    // Otherwise, add new token.
                    Ok(ttype) => {
                        tokens.push(self.build_token(&prev_state, ttype)?);
                    }
                    // Invalid token type. Ignore.
                    _ => {}
                }

                // Reset state to before next numeric token.
                // We do this because may need context of next token for numeric token.
                if next_digit_token.is_some() {
                    *state = prev_state
                }
            }
            // Itemname at end of punctuation/statement.
            // ex. Dog with Chili.
            (Some(_), true) | (None, true) => {
                let word = self.get_text(state, false).ok();
                // If LogicType::With prev token type, assume food.
                let ttype = if matches!(
                    tokens.last().map(|t| &t.ttype),
                    Some(TokenType::Logic(LogicType::With))
                ) {
                    TokenType::Entity(EntityType::Food {
                        number: None,
                        name: word,
                    })
                } else {
                    TokenType::Entity(EntityType::Pet {
                        number: None,
                        name: word,
                        attr: None,
                    })
                };
                tokens.push(self.build_token(state, ttype)?)
            }
            // Any non-itemname word token.
            (Some(_), false) | (None, false) => {
                let word = self.get_text(state, true)?;
                if let Ok(ttype) = TokenType::parse(word, None) {
                    tokens.push(self.build_token(state, ttype)?);
                }
            }
        }

        Ok(())
    }

    /// Scans numeric tokens that being with a `+` or `-`.
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
                let Some(token) =
                    self.consume_while_cond(state, Some(num_literal_state), 1, is_alpha)
                else {
                    bail!("{state} No attribute after signed numerical characters.")
                };
                tokens.push(token)
            }
            // Percentage multiplier.
            // ex. +100% trumpets
            Some('%') => {
                let Some(mut token) =
                    self.consume_while_cond(state, Some(num_literal_state), 2, is_alpha)
                else {
                    bail!("{state} No attribute after signed numerical characters.")
                };
                if let TokenType::Entity(ref mut attr_type) = token.ttype {
                    *attr_type = attr_type.clone().into_percent_variant()?;
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

    /// Scans numeric tokens starting with a digit.
    fn scan_numeric_token(
        &'src self,
        state: &mut Scanner,
        tokens: &mut Vec<Token<'src>>,
    ) -> anyhow::Result<()> {
        // Keep going if digit. ex. '12/12'
        while self.advance_by_cond(state, is_digit).is_some() {}

        let num_literal_state = state.clone();
        let next_char = self.peek(state.current);
        match next_char {
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
                let mut health_token = self
                    .consume_while_cond(state, None, 1, is_digit)
                    .with_context(|| format!("{state} No health after summon stats '/'."))?;
                health_token.ttype =
                    TokenType::Entity(EntityType::Health(Some(health_token.text.parse()?)));
                tokens.push(health_token)
            }
            // ex. 1 attack
            // ex. 1-gold
            Some(' ') | Some('-') | Some('%') => {
                let num_literal_token = self.build_token(
                    &num_literal_state,
                    TokenType::Numeric(NumericType::Number(Some(
                        self.get_text(&num_literal_state, false)?.parse()?,
                    ))),
                )?;
                // Adjust cursor based on next char.
                let cur_adj = match next_char.map(|chr| chr.is_whitespace() || chr == '-') {
                    Some(true) | None => 1,
                    Some(false) => 2,
                };
                let Some(mut next_token) =
                    self.consume_while_cond(state, Some(num_literal_state), cur_adj, is_alpha)
                else {
                    return Ok(());
                };
                let is_perc_token = next_char.as_ref().map_or(false, |chr| *chr == '%');
                match (&mut next_token.ttype, is_perc_token) {
                    // Try to turn into percent variant if percent next token.
                    (TokenType::Entity(attr_type), true) => {
                        *attr_type = attr_type.clone().into_percent_variant()?;
                        tokens.push(next_token);
                    }
                    // Only add num attr token if next token related to entities.
                    (TokenType::Numeric(_) | TokenType::Entity(_), _) => tokens.push(next_token),
                    // Otherwise, just add number token.
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
    ) -> Option<Token<'src>> {
        state.move_cursor(true, cur_adj).set_start_to_current();

        // Move cursor while condition is met.
        while self.advance_by_cond(state, &cond).is_some() {}

        let Ok(word) = self.get_text(state, true) else {
            return None;
        };
        if let Some(mut updated_literal_state) = literal_state {
            let literal_value = self.get_text(&updated_literal_state, false).ok();
            // Use literal state updated so Token text includes both literal value and attribute token.
            updated_literal_state.current = state.current;
            let Some(ttype) = TokenType::parse(word, literal_value).ok() else {
                return None;
            };
            self.build_token(&updated_literal_state, ttype).ok()
        } else {
            let Ok(ttype) = TokenType::parse(word, None) else {
                return None;
            };
            self.build_token(state, ttype).ok()
        }
    }

    /// Gets text slice but with [`Scanner`].
    /// * TODO: Combine with [`SAPText::get_text_slice`].
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

    /// Gets text slice.
    ///
    /// ### Params
    /// * `i`
    ///     * Generic with [`SliceIndex<str>`].
    ///     * Passed to [`str::get`].
    /// * `lowercase`
    ///
    /// ### Returns
    /// * Slice of source text.
    fn get_text_slice<I>(&'src self, i: I, lowercase: bool) -> anyhow::Result<&I::Output>
    where
        I: SliceIndex<str>,
    {
        let source = if lowercase {
            &self.lowercase_effect
        } else {
            self.effect
        };
        source.get(i).context("Invalid indices in source text.")
    }

    /// Build a token.
    ///
    /// ### Params
    /// * `state`
    ///     * [`Scanner`] containing indices of token text
    /// * `ttype`
    ///     * [`TokenType`] of token text.
    ///
    /// ### Returns
    /// * New [`Token`],
    fn build_token(&'src self, state: &Scanner, ttype: TokenType<'src>) -> anyhow::Result<Token> {
        Ok(Token {
            ttype,
            text: self.get_text(state, false)?,
            metadata: state.clone(),
        })
    }

    /// Advances [`Scanner`] one character.
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
    fn test_tokenize_pet_with_attr() {
        let txt = SAPText::new("If a random Strawberry pet, gain +2 attack.");
        let tokens = txt.tokenize().unwrap();

        assert_eq!(
            *tokens,
            [
                Token {
                    ttype: TokenType::Logic(LogicType::If),
                    text: "If",
                    metadata: Scanner {
                        start: 0,
                        current: 2,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Position(PositionType::Any),
                    text: "random",
                    metadata: Scanner {
                        start: 5,
                        current: 11,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Entity(EntityType::Pet {
                        number: None,
                        name: None,
                        attr: Some("Strawberry")
                    }),
                    text: "Strawberry pet",
                    metadata: Scanner {
                        start: 12,
                        current: 26,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Action(ActionType::Gain),
                    text: "gain",
                    metadata: Scanner {
                        start: 28,
                        current: 32,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Entity(EntityType::Attack(Some(2))),
                    text: "+2 attack",
                    metadata: Scanner {
                        start: 33,
                        current: 42,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::EndText,
                    text: "",
                    metadata: Scanner {
                        start: 43,
                        current: 43,
                        line: 1
                    }
                }
            ]
        )
    }

    #[test]
    fn test_tokenize_three_word_itemname() {
        let txt = SAPText::new("Gain Fortune Cookie Perk");
        let tokens = txt.tokenize().unwrap();
        assert_eq!(
            *tokens,
            [
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
                    ttype: TokenType::Entity(EntityType::Food {
                        number: None,
                        name: Some("Fortune Cookie Perk")
                    }),
                    text: "Fortune Cookie Perk",
                    metadata: Scanner {
                        start: 5,
                        current: 24,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::EndText,
                    text: "",
                    metadata: Scanner {
                        start: 24,
                        current: 24,
                        line: 1
                    }
                }
            ]
        )
    }

    #[test]
    fn test_tokenize_front_itemname() {
        let invalid_name_at_front = SAPText::new("Beluga Sturgeon");
        let invalid_name_at_tokens = invalid_name_at_front.tokenize().unwrap();
        // Names will be mangled if at front. Assumes must have some word before.
        assert_eq!(
            *invalid_name_at_tokens,
            [
                Token {
                    ttype: TokenType::Entity(EntityType::Pet {
                        number: None,
                        name: Some("Sturgeon"),
                        attr: None
                    }),
                    text: "Sturgeon",
                    metadata: Scanner {
                        start: 7,
                        current: 15,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::EndText,
                    text: "",
                    metadata: Scanner {
                        start: 15,
                        current: 15,
                        line: 1
                    }
                }
            ]
        );
    }

    #[test]
    fn test_tokenize_pet_with_food_itemname() {
        // ex. Bus with Chili.
        let txt = SAPText::new("Summon one 5/5 Bus with Chili.");
        let tokens = txt.tokenize().unwrap();

        assert_eq!(
            *tokens,
            [
                Token {
                    ttype: TokenType::Action(ActionType::Summon),
                    text: "Summon",
                    metadata: Scanner {
                        start: 0,
                        current: 6,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Numeric(NumericType::Number(Some(1))),
                    text: "one",
                    metadata: Scanner {
                        start: 7,
                        current: 10,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Entity(EntityType::Attack(Some(5))),
                    text: "5",
                    metadata: Scanner {
                        start: 11,
                        current: 12,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Entity(EntityType::Health(Some(5))),
                    text: "5",
                    metadata: Scanner {
                        start: 13,
                        current: 14,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Entity(EntityType::Pet {
                        number: None,
                        name: Some("Bus"),
                        attr: None
                    }),
                    text: "Bus",
                    metadata: Scanner {
                        start: 15,
                        current: 18,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Logic(LogicType::With),
                    text: "with",
                    metadata: Scanner {
                        start: 19,
                        current: 23,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Entity(EntityType::Food {
                        number: None,
                        name: Some("Chili")
                    }),
                    text: "Chili",
                    metadata: Scanner {
                        start: 24,
                        current: 29,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::EndText,
                    text: "",
                    metadata: Scanner {
                        start: 30,
                        current: 30,
                        line: 1
                    }
                }
            ]
        );
    }

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
                    ttype: TokenType::EndText,
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
                    ttype: TokenType::EndText,
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
                    ttype: TokenType::EndText,
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
                    ttype: TokenType::EndText,
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
