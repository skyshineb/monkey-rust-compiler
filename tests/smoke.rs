use monkey_rust_compiler::builtins::builtin_names;
use monkey_rust_compiler::lexer::Lexer;
use monkey_rust_compiler::position::Position;
use monkey_rust_compiler::token::TokenKind;

#[test]
fn position_default_is_one_based() {
    let pos = Position::default();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.col, 1);
    assert_eq!(pos.to_string(), "1:1");
}

#[test]
fn lexer_placeholder_returns_eof() {
    let mut lexer = Lexer::new("let x = 1;");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Eof);
}

#[test]
fn builtin_names_match_contract_set() {
    let names = builtin_names();
    assert_eq!(names, ["len", "first", "last", "rest", "push", "puts"]);
}
