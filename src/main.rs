// https://craftinginterpreters.com/scanning.html

use std::fmt::Display;

pub mod token;

use anyhow::bail;
use token::{Token, TokenType};

fn is_digit(chr: Option<char>) -> Option<char> {
    chr.filter(|chr| chr.is_ascii_digit())
}

fn is_alpha(chr: Option<char>) -> Option<char> {
    chr.filter(|chr| chr.is_alphabetic())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannerState {
    /// Start character index of lexeme.
    pub start: usize,
    /// Current character index of lexeme.
    pub current: usize,
    /// Curent line
    pub line: usize,
}

impl ScannerState {
    /// Move [`ScannerState::current`] cursor index by some amount.
    ///
    /// ### Params
    /// * `curr`
    ///     * If `true`, current cursor. Otherwise, start cursor.
    /// * `by`
    ///     * Amount to move cursor by.
    ///     * If negative, will perform saturating sub.
    ///
    /// ### Returns
    /// * Instance
    fn move_cursor(&mut self, curr: bool, by: isize) -> &mut Self {
        let cursor = if curr {
            &mut self.current
        } else {
            &mut self.start
        };
        if by.is_negative() {
            *cursor = cursor.saturating_sub(-by as usize);
        } else {
            *cursor += by as usize;
        }
        self
    }

    /// Set [`ScannerState::start`] to [`ScannerState::current`].
    fn start_to_current(&mut self) -> &mut Self {
        self.start = self.current;
        self
    }
}

impl Display for ScannerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {} ({}-{})", self.line, self.start, self.current)
    }
}

impl Default for ScannerState {
    fn default() -> Self {
        Self {
            start: Default::default(),
            current: Default::default(),
            line: 1,
        }
    }
}

#[derive(Default)]
struct Scanner<'src> {
    source: &'src str,
}

impl<'src> Scanner<'src> {
    fn new(source: &'src str) -> Scanner<'src> {
        Scanner { source }
    }

    fn advance(&self, state: &mut ScannerState) -> Option<char> {
        if let Some(char) = self.peek(state.current) {
            state.current += 1;
            Some(char)
        } else {
            None
        }
    }

    pub fn tokenize(&'src self) -> anyhow::Result<Vec<Token<'src>>> {
        let mut tokens = vec![];
        let mut state = ScannerState::default();

        loop {
            state.start = state.current;
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
        Ok(tokens)
    }

    fn scan_token(
        &'src self,
        state: &mut ScannerState,
        tokens: &mut Vec<Token<'src>>,
    ) -> anyhow::Result<Option<()>> {
        let Some(c) = self.advance(state) else {
            return Ok(None);
        };

        match c {
            'A'..='Z' | 'a'..='z' => {
                // Match actions (gain, damage)

                // Match numbers (one, two, three)

                // Match positions (random, any, all)

                // Match target (friend (+plural))

                // Match item (foodname, petname)

                // Match level
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
                eprintln!("{state}. Invalid character ({c})")
            }
        }

        Ok(Some(()))
    }

    fn scan_sign_token(
        &'src self,
        state: &mut ScannerState,
        tokens: &mut Vec<Token<'src>>,
    ) -> anyhow::Result<()> {
        state.start_to_current();

        // Keep reading until not a digit.
        // state.current now points to char after digits.
        while self.advance_by_cond(state, is_digit).is_some() {}

        // Include sign in value.
        let mut num_scan_state = state.clone();
        num_scan_state.move_cursor(false, -1);

        let next_chr = self.peek(state.current);
        match next_chr {
            // Space between num and attribute
            // ex. +1 attack
            Some(' ') => {
                state.move_cursor(true, 1).start_to_current();

                // Consume entire alphabetic string ahead.
                while self.advance_by_cond(state, is_alpha).is_some() {}

                let ttype: TokenType = self.source.get(state.start..state.current).try_into()?;
                tokens.push(self.build_token(&num_scan_state, ttype))
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
        state: &mut ScannerState,
        tokens: &mut Vec<Token<'src>>,
    ) -> anyhow::Result<()> {
        // Keep going if digit. ex. '12/12'
        while self.advance_by_cond(state, is_digit).is_some() {}

        match self.peek(state.current) {
            // ex. 12/12
            Some('/') => {
                tokens.push(self.build_token(state, TokenType::Attack));

                // Set new cursor position skipping delimiter.
                state.move_cursor(true, 1).start_to_current();

                // Keep going after '/' to get rest.
                while self.advance_by_cond(state, is_digit).is_some() {}

                // Didn't move cursor.
                if state.current == state.start {
                    bail!(
                        "{state}. Character ({:?}) following '/' not a digit.",
                        self.peek(state.current + 1)
                    );
                };
                tokens.push(self.build_token(state, TokenType::Health))
            }
            // ex. 1-gold
            Some('-') => {
                // Store state so only numeric values captured.
                let num_scan_state = state.clone();

                state.move_cursor(true, 1).start_to_current();

                // Move cursor while alphabetical chars.
                while self.advance_by_cond(state, is_alpha).is_some() {}

                let ttype: TokenType = self.source.get(state.start..state.current).try_into()?;

                tokens.push(self.build_token(&num_scan_state, ttype))
            }
            // ex. 1 attack
            Some(' ') => {
                state.move_cursor(true, 1).start_to_current();
                while self.advance_by_cond(state, is_alpha).is_some() {}
            }
            // Ignore everything else and just add number.
            Some(_) | None => tokens.push(self.build_token(state, TokenType::Number)),
        }

        Ok(())
    }

    /// Peek at index character without advancing `Scanner`.
    fn peek(&self, idx: usize) -> Option<char> {
        self.source
            .as_bytes()
            .get(idx)
            .filter(|byte| byte.is_ascii())
            .map(|byte| *byte as char)
    }

    fn build_token(&self, state: &ScannerState, ttype: TokenType) -> Token {
        let Some(text) = self.source.get(state.start..state.current) else {
            panic!(
                "Invalid start {} and current {} indices in source.",
                state.start, state.current
            )
        };
        Token {
            ttype,
            text,
            metadata: state.clone(),
        }
    }

    /// Conditional [`Scanner::advance`].
    ///
    /// ### Params
    /// * `state`
    ///     * Current state of [`Scanner`].
    /// * `pass_cond`
    ///     * Closure taking the next character and return an optional char.
    ///     * Passes if [`Option::Some`] and increments `state.current` char position.
    ///
    /// ### Returns
    /// * Next character or [`Option::None`].
    fn advance_by_cond(
        &self,
        state: &mut ScannerState,
        pass_cond: impl Fn(Option<char>) -> Option<char>,
    ) -> Option<char> {
        if self.is_at_end(state) {
            return None;
        }

        if let Some(chr) = pass_cond(self.peek(state.current)) {
            state.current += 1;
            Some(chr)
        } else {
            None
        }
    }

    fn is_at_end(&self, state: &ScannerState) -> bool {
        state.current >= self.source.len()
    }
}

fn main() {}

mod test {
    use super::*;

    #[test]
    fn test_tokenize_sign_numeric_attr() {
        let valid_attr_num = Scanner::new("+13 health and +12 attack");
        let tokens = valid_attr_num.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token {
                    ttype: TokenType::Health,
                    text: "+13",
                    metadata: ScannerState {
                        start: 0,
                        current: 3,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Attack,
                    text: "+12",
                    metadata: ScannerState {
                        start: 15,
                        current: 18,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::End,
                    text: "",
                    metadata: ScannerState {
                        start: 25,
                        current: 25,
                        line: 1
                    }
                }
            ]
        )
    }

    #[test]
    fn test_tokenize_numeric_attr() {
        let valid_attr_num = Scanner::new("1-gold");
        let tokens = valid_attr_num.tokenize().unwrap();

        assert_eq!(
            tokens,
            vec![
                Token {
                    ttype: TokenType::Gold,
                    text: "1",
                    metadata: ScannerState {
                        start: 0,
                        current: 1,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::End,
                    text: "",
                    metadata: ScannerState {
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
        let valid_summon_stats: Scanner = Scanner::new("12/13");
        let invalid_summon_stats_health_missing: Scanner = Scanner::new("12/");
        let invalid_summon_stats_health_nondigit: Scanner = Scanner::new("12/a");

        let tokens = valid_summon_stats.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token {
                    ttype: TokenType::Attack,
                    text: "12",
                    metadata: ScannerState {
                        start: 0,
                        current: 2,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::Health,
                    text: "13",
                    metadata: ScannerState {
                        start: 3,
                        current: 5,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::End,
                    text: "",
                    metadata: ScannerState {
                        start: 5,
                        current: 5,
                        line: 1
                    }
                }
            ]
        )
    }
}
