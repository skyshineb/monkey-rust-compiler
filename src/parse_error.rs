use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::position::Position;

/// Parser error with source position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub pos: Position,
}

impl ParseError {
    pub fn new(pos: Position, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            pos,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}: {}", self.pos, self.message)
    }
}
