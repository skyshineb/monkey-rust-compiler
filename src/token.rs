use crate::position::Position;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Token kinds recognized by the Monkey language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    pub pos: Position,
}

impl Token {
    pub fn new(kind: TokenKind, literal: impl Into<String>, pos: Position) -> Self {
        Self {
            kind,
            literal: literal.into(),
            pos,
        }
    }
}

/// Resolve identifier text to keyword tokens when applicable.
pub fn lookup_ident(ident: &str) -> TokenKind {
    // TODO(step-3): lexer should call this for identifier token classification.
    match ident {
        "fn" => TokenKind::Function,
        "let" => TokenKind::Let,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "return" => TokenKind::Return,
        "while" => TokenKind::While,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        _ => TokenKind::Ident,
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let name = match self {
            TokenKind::Illegal => "Illegal",
            TokenKind::Eof => "Eof",
            TokenKind::Ident => "Ident",
            TokenKind::Int => "Int",
            TokenKind::String => "String",
            TokenKind::Assign => "Assign",
            TokenKind::Plus => "Plus",
            TokenKind::Minus => "Minus",
            TokenKind::Bang => "Bang",
            TokenKind::Asterisk => "Asterisk",
            TokenKind::Slash => "Slash",
            TokenKind::Lt => "Lt",
            TokenKind::Gt => "Gt",
            TokenKind::Eq => "Eq",
            TokenKind::NotEq => "NotEq",
            TokenKind::Le => "Le",
            TokenKind::Ge => "Ge",
            TokenKind::And => "And",
            TokenKind::Or => "Or",
            TokenKind::Comma => "Comma",
            TokenKind::Semicolon => "Semicolon",
            TokenKind::Colon => "Colon",
            TokenKind::LParen => "LParen",
            TokenKind::RParen => "RParen",
            TokenKind::LBrace => "LBrace",
            TokenKind::RBrace => "RBrace",
            TokenKind::LBracket => "LBracket",
            TokenKind::RBracket => "RBracket",
            TokenKind::Function => "Function",
            TokenKind::Let => "Let",
            TokenKind::True => "True",
            TokenKind::False => "False",
            TokenKind::If => "If",
            TokenKind::Else => "Else",
            TokenKind::Return => "Return",
            TokenKind::While => "While",
            TokenKind::Break => "Break",
            TokenKind::Continue => "Continue",
        };

        write!(f, "{name}")
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}(\"{}\") @ {}", self.kind, self.literal, self.pos)
    }
}
