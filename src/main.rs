// https://craftinginterpreters.com/scanning.html

use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenType {
    Action,
    Number,
    Position,
    Target,

    // Stats
    Attack,
    Health,
    Level,
    Experience,
    SummonAttack,
    SummonHealth,

    // Entity
    Trumpet,
    Strawberry,

    /// End of statement
    End,

    // Positions
    Adjacent,

    // Targets
    Perks,
    Food,
    Friendly,

    // Conditions/Logic
    If,
    Your,
    Equal,

    // Actions
    Choose,
    Deal,
    Gain,
    Give,
    Push,
    Remove,
    Set,
    Spend,
    Stock,
    Summon,
    Swap,
    Break,
    Copy,
    Make,
    Friend,
    Increase,
    Resummon,
    Steal,
    Activate,
    Discount,
    Knock,
    Reduce,
    Swallow,
    Take,
    Transform,
    Replace,
    Shuffle,
    Unfreeze,
}

fn is_digit(chr: Option<char>) -> Option<char> {
    chr.filter(|chr| chr.is_ascii_digit())
}

fn is_alpha(chr: Option<char>) -> Option<char> {
    chr.filter(|chr| chr.is_alphabetic())
}

#[derive(Debug, PartialEq, Eq)]
struct Token<'src> {
    /// Type of token.
    ttype: TokenType,
    /// Text of token.
    text: &'src str,
    /// Token source metadata.
    metadata: ScannerState,
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?}) ({})", self.metadata, self.ttype, self.text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScannerState {
    /// Start character index of lexeme.
    start: usize,
    /// Current character index of lexeme.
    current: usize,
    /// Curent line
    line: usize,
}

impl ScannerState {
    /// Move [`ScannerState::current`] cursor index by some amount.
    ///
    /// ### Params
    /// * `by`
    ///     * Amount to move cursor by.
    ///     * If negative, will perform saturating sub.
    ///
    /// ### Returns
    /// * Instance
    fn move_cursor(&mut self, by: isize) -> &mut Self {
        if by.is_negative() {
            self.current = self.current.saturating_sub(-by as usize);
        } else {
            self.current += by as usize;
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

    pub fn tokenize(&'src self) -> Vec<Token<'src>> {
        let mut tokens = vec![];
        let mut state = ScannerState::default();

        loop {
            state.start = state.current;
            if self.scan_token(&mut state, &mut tokens).is_none() {
                break;
            };
        }

        // End of statement.
        tokens.push(Token {
            ttype: TokenType::End,
            text: "",
            metadata: state,
        });
        tokens
    }

    fn scan_token(
        &'src self,
        state: &mut ScannerState,
        tokens: &mut Vec<Token<'src>>,
    ) -> Option<()> {
        let Some(c) = self.advance(state) else {
            return None;
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
                self.scan_sign_token(state, tokens);
            }
            '\n' => {
                state.line += 1;
            }
            // Skip punctuation.
            '.' | ',' | ' ' | '\t' | '/' => {}
            // Scan digits.
            '0'..='9' => {
                self.scan_numeric_token(state, tokens);
            }
            _ => {
                eprintln!("{state}. Invalid character ({c})")
            }
        }

        Some(())
    }

    fn scan_sign_token(
        &'src self,
        state: &mut ScannerState,
        tokens: &mut Vec<Token<'src>>,
    ) -> Option<()> {
        // Ignore +
        state.move_cursor(1).start_to_current();

        // Keep reading until not a digit.
        // state.current now points to char after digits.
        while self.advance_by_cond(state, is_digit).is_some() {}

        // Space between num and attribute (ex. +1 attack)
        let next_chr = self.peek(state.current);
        if next_chr.filter(|chr| *chr == ' ').is_some() {
            state.move_cursor(1).start_to_current();

            // Consume entire alphabetic string ahead.
            while self.advance_by_cond(state, is_alpha).is_some() {}
        } else {
            eprintln!("{state} Non-whitespace {next_chr:?} after digit.");
            return None;
        }
        Some(())
    }

    fn scan_numeric_token(
        &'src self,
        state: &mut ScannerState,
        tokens: &mut Vec<Token<'src>>,
    ) -> Option<()> {
        // Keep going if digit. ex. '12/12'
        while self.advance_by_cond(state, is_digit).is_some() {}

        // Check if on '/' delimiting summon stats (atk/health).
        if self.peek(state.current).filter(|chr| *chr == '/').is_some() {
            tokens.push(self.build_token(state, TokenType::SummonAttack));

            // Set new cursor position skipping delimiter.
            state.move_cursor(1).start_to_current();

            // Keep going after '/' to get rest.
            while self.advance_by_cond(state, is_digit).is_some() {}

            // Didn't move cursor.
            if state.current == state.start {
                eprintln!(
                    "{state}. Character ({:?}) following '/' not a digit.",
                    self.peek(state.current + 1)
                );
                return Some(());
            };
            tokens.push(self.build_token(state, TokenType::SummonHealth))
        } else {
            // Otherwise, just a number.
            tokens.push(self.build_token(state, TokenType::Number))
        }
        Some(())
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
    fn test_tokenize_numeric() {
        let valid: Scanner = Scanner::new("12/13");
        let invalid_health_missing: Scanner = Scanner::new("12/");
        let invalid_health_nondigit: Scanner = Scanner::new("12/a");

        let tokens = valid.tokenize();
        assert_eq!(
            tokens,
            vec![
                Token {
                    ttype: TokenType::SummonAttack,
                    text: "12",
                    metadata: ScannerState {
                        start: 0,
                        current: 2,
                        line: 1
                    }
                },
                Token {
                    ttype: TokenType::SummonHealth,
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
