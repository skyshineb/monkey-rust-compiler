use crate::position::Position;

/// Token kinds recognized by the Monkey language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Illegal,
    Eof,

    Ident,
    Int,
    String,

    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,

    Lt,
    Gt,
    Eq,
    NotEq,
    Le,
    Ge,
    And,
    Or,

    Comma,
    Semicolon,
    Colon,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
    While,
    Break,
    Continue,
}

/// Token with literal text and source position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: String,
    pub position: Position,
}

impl Token {
    pub fn new(kind: TokenKind, literal: impl Into<String>, position: Position) -> Self {
        Self {
            kind,
            literal: literal.into(),
            position,
        }
    }
}
