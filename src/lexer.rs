use crate::position::Position;
use crate::token::{Token, TokenKind};

/// Lexer placeholder.
#[derive(Debug, Clone)]
pub struct Lexer {
    input: String,
}

impl Lexer {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
        }
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn next_token(&mut self) -> Token {
        // TODO(step-2): implement real token scanning with positions.
        Token::new(TokenKind::Eof, "", Position::default())
    }
}
