use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::position::Position;

/// Parser error placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub position: Option<Position>,
}

impl ParseError {
    pub fn new(message: impl Into<String>, position: Option<Position>) -> Self {
        Self {
            message: message.into(),
            position,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.position {
            Some(pos) => write!(f, "{pos}: {}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}
