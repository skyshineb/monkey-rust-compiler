use std::fmt::{Display, Formatter, Result as FmtResult};

/// Source position (1-based line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl Default for Position {
    /// Default source position at start-of-input.
    fn default() -> Self {
        // TODO(step-3): lexer should attach accurate token positions while scanning.
        Self { line: 1, col: 1 }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}:{}", self.line, self.col)
    }
}
