use monkey_rust_compiler::position::Position;
use monkey_rust_compiler::token::{lookup_ident, Token, TokenKind};

#[test]
fn position_new_stores_line_and_col() {
    let pos = Position::new(3, 14);
    assert_eq!(pos.line, 3);
    assert_eq!(pos.col, 14);
}

#[test]
fn position_equality_works() {
    assert_eq!(Position::new(2, 5), Position::new(2, 5));
    assert_ne!(Position::new(2, 5), Position::new(2, 6));
}

#[test]
fn position_display_is_deterministic() {
    assert_eq!(Position::new(10, 2).to_string(), "10:2");
}

#[test]
fn token_new_stores_kind_literal_and_pos() {
    let pos = Position::new(1, 7);
    let token = Token::new(TokenKind::Let, "let", pos);

    assert_eq!(token.kind, TokenKind::Let);
    assert_eq!(token.literal, "let");
    assert_eq!(token.pos, pos);
}

#[test]
fn token_equality_compares_all_fields() {
    let left = Token::new(TokenKind::Int, "1", Position::new(1, 1));
    let same = Token::new(TokenKind::Int, "1", Position::new(1, 1));
    let different_kind = Token::new(TokenKind::Ident, "1", Position::new(1, 1));
    let different_literal = Token::new(TokenKind::Int, "2", Position::new(1, 1));
    let different_pos = Token::new(TokenKind::Int, "1", Position::new(1, 2));

    assert_eq!(left, same);
    assert_ne!(left, different_kind);
    assert_ne!(left, different_literal);
    assert_ne!(left, different_pos);
}

#[test]
fn token_display_is_deterministic() {
    let token = Token::new(TokenKind::Let, "let", Position::new(1, 1));
    assert_eq!(token.to_string(), "Let(\"let\") @ 1:1");
}

#[test]
fn lookup_ident_maps_keywords() {
    let cases = [
        ("fn", TokenKind::Function),
        ("let", TokenKind::Let),
        ("true", TokenKind::True),
        ("false", TokenKind::False),
        ("if", TokenKind::If),
        ("else", TokenKind::Else),
        ("return", TokenKind::Return),
        ("while", TokenKind::While),
        ("break", TokenKind::Break),
        ("continue", TokenKind::Continue),
    ];

    for (input, expected) in cases {
        assert_eq!(lookup_ident(input), expected, "input={input}");
    }
}

#[test]
fn lookup_ident_non_keywords_default_to_ident() {
    let cases = ["foobar", "x", "_tmp"];

    for input in cases {
        assert_eq!(lookup_ident(input), TokenKind::Ident, "input={input}");
    }
}
