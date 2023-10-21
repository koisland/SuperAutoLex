use std::fmt::Display;

/// [`SAPText`] parser state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scanner {
    /// Start character index of lexeme.
    pub start: usize,
    /// Current character index of lexeme.
    pub current: usize,
    /// Current line.
    pub line: usize,
}

impl Scanner {
    /// Move [`Scanner::current`] cursor index by some amount.
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
    pub(crate) fn move_cursor(&mut self, curr: bool, by: isize) -> &mut Self {
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

    /// Set [`Scanner::start`] to [`Scanner::current`].
    pub(crate) fn set_start_to_current(&mut self) -> &mut Self {
        self.start = self.current;
        self
    }
}

impl Display for Scanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {} ({}-{})", self.line, self.start, self.current)
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            start: Default::default(),
            current: Default::default(),
            line: 1,
        }
    }
}
