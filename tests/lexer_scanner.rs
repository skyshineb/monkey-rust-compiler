use monkey_rust_compiler::lexer::Lexer;
use monkey_rust_compiler::position::Position;
use monkey_rust_compiler::token::TokenKind;

fn collect(input: &str) -> Vec<(TokenKind, String, Position)> {
    Lexer::new(input)
        .tokenize_all()
        .into_iter()
        .map(|t| (t.kind, t.literal, t.pos))
        .collect()
}

#[test]
fn basic_token_stream_covers_protocol_surface() {
    let input = r#"
let five = 5;
let ten = 10;
let add = fn(x, y) { x + y; };
let arr = [1, 2, 3];
let hash = {"x": 1, "y": 2};
if (five < ten) { return add(five, ten); } else { while (true) { break; continue; } }
"done"
"#;

    let got: Vec<(TokenKind, String)> =
        collect(input).into_iter().map(|(k, l, _)| (k, l)).collect();

    let expected = vec![
        (TokenKind::Let, "let".to_string()),
        (TokenKind::Ident, "five".to_string()),
        (TokenKind::Assign, "=".to_string()),
        (TokenKind::Int, "5".to_string()),
        (TokenKind::Semicolon, ";".to_string()),
        (TokenKind::Let, "let".to_string()),
        (TokenKind::Ident, "ten".to_string()),
        (TokenKind::Assign, "=".to_string()),
        (TokenKind::Int, "10".to_string()),
        (TokenKind::Semicolon, ";".to_string()),
        (TokenKind::Let, "let".to_string()),
        (TokenKind::Ident, "add".to_string()),
        (TokenKind::Assign, "=".to_string()),
        (TokenKind::Function, "fn".to_string()),
        (TokenKind::LParen, "(".to_string()),
        (TokenKind::Ident, "x".to_string()),
        (TokenKind::Comma, ",".to_string()),
        (TokenKind::Ident, "y".to_string()),
        (TokenKind::RParen, ")".to_string()),
        (TokenKind::LBrace, "{".to_string()),
        (TokenKind::Ident, "x".to_string()),
        (TokenKind::Plus, "+".to_string()),
        (TokenKind::Ident, "y".to_string()),
        (TokenKind::Semicolon, ";".to_string()),
        (TokenKind::RBrace, "}".to_string()),
        (TokenKind::Semicolon, ";".to_string()),
        (TokenKind::Let, "let".to_string()),
        (TokenKind::Ident, "arr".to_string()),
        (TokenKind::Assign, "=".to_string()),
        (TokenKind::LBracket, "[".to_string()),
        (TokenKind::Int, "1".to_string()),
        (TokenKind::Comma, ",".to_string()),
        (TokenKind::Int, "2".to_string()),
        (TokenKind::Comma, ",".to_string()),
        (TokenKind::Int, "3".to_string()),
        (TokenKind::RBracket, "]".to_string()),
        (TokenKind::Semicolon, ";".to_string()),
        (TokenKind::Let, "let".to_string()),
        (TokenKind::Ident, "hash".to_string()),
        (TokenKind::Assign, "=".to_string()),
        (TokenKind::LBrace, "{".to_string()),
        (TokenKind::String, "x".to_string()),
        (TokenKind::Colon, ":".to_string()),
        (TokenKind::Int, "1".to_string()),
        (TokenKind::Comma, ",".to_string()),
        (TokenKind::String, "y".to_string()),
        (TokenKind::Colon, ":".to_string()),
        (TokenKind::Int, "2".to_string()),
        (TokenKind::RBrace, "}".to_string()),
        (TokenKind::Semicolon, ";".to_string()),
        (TokenKind::If, "if".to_string()),
        (TokenKind::LParen, "(".to_string()),
        (TokenKind::Ident, "five".to_string()),
        (TokenKind::Lt, "<".to_string()),
        (TokenKind::Ident, "ten".to_string()),
        (TokenKind::RParen, ")".to_string()),
        (TokenKind::LBrace, "{".to_string()),
        (TokenKind::Return, "return".to_string()),
        (TokenKind::Ident, "add".to_string()),
        (TokenKind::LParen, "(".to_string()),
        (TokenKind::Ident, "five".to_string()),
        (TokenKind::Comma, ",".to_string()),
        (TokenKind::Ident, "ten".to_string()),
        (TokenKind::RParen, ")".to_string()),
        (TokenKind::Semicolon, ";".to_string()),
        (TokenKind::RBrace, "}".to_string()),
        (TokenKind::Else, "else".to_string()),
        (TokenKind::LBrace, "{".to_string()),
        (TokenKind::While, "while".to_string()),
        (TokenKind::LParen, "(".to_string()),
        (TokenKind::True, "true".to_string()),
        (TokenKind::RParen, ")".to_string()),
        (TokenKind::LBrace, "{".to_string()),
        (TokenKind::Break, "break".to_string()),
        (TokenKind::Semicolon, ";".to_string()),
        (TokenKind::Continue, "continue".to_string()),
        (TokenKind::Semicolon, ";".to_string()),
        (TokenKind::RBrace, "}".to_string()),
        (TokenKind::RBrace, "}".to_string()),
        (TokenKind::String, "done".to_string()),
        (TokenKind::Eof, "".to_string()),
    ];

    assert_eq!(got, expected);
}

#[test]
fn multi_character_operators_are_single_tokens() {
    let got: Vec<(TokenKind, String)> = collect("== != <= >= && ||")
        .into_iter()
        .map(|(k, l, _)| (k, l))
        .collect();

    assert_eq!(
        got,
        vec![
            (TokenKind::Eq, "==".to_string()),
            (TokenKind::NotEq, "!=".to_string()),
            (TokenKind::Le, "<=".to_string()),
            (TokenKind::Ge, ">=".to_string()),
            (TokenKind::And, "&&".to_string()),
            (TokenKind::Or, "||".to_string()),
            (TokenKind::Eof, "".to_string()),
        ]
    );
}

#[test]
fn comments_are_skipped() {
    let input = "# full line\nlet x = 1; # trailing\nlet y = 2;";
    let got: Vec<(TokenKind, String)> =
        collect(input).into_iter().map(|(k, l, _)| (k, l)).collect();

    assert_eq!(
        got,
        vec![
            (TokenKind::Let, "let".to_string()),
            (TokenKind::Ident, "x".to_string()),
            (TokenKind::Assign, "=".to_string()),
            (TokenKind::Int, "1".to_string()),
            (TokenKind::Semicolon, ";".to_string()),
            (TokenKind::Let, "let".to_string()),
            (TokenKind::Ident, "y".to_string()),
            (TokenKind::Assign, "=".to_string()),
            (TokenKind::Int, "2".to_string()),
            (TokenKind::Semicolon, ";".to_string()),
            (TokenKind::Eof, "".to_string()),
        ]
    );
}

#[test]
fn tracks_positions_for_key_tokens() {
    let input = "let x = 1;\nfoo == \"bar\"\n";
    let got = collect(input);

    assert_eq!(got[0].0, TokenKind::Let);
    assert_eq!(got[0].2, Position::new(1, 1));

    let foo = got
        .iter()
        .find(|(k, l, _)| *k == TokenKind::Ident && l == "foo");
    assert_eq!(foo.map(|(_, _, p)| *p), Some(Position::new(2, 1)));

    let eqeq = got.iter().find(|(k, _, _)| *k == TokenKind::Eq);
    assert_eq!(eqeq.map(|(_, _, p)| *p), Some(Position::new(2, 5)));

    let string = got
        .iter()
        .find(|(k, l, _)| *k == TokenKind::String && l == "bar");
    assert_eq!(string.map(|(_, _, p)| *p), Some(Position::new(2, 8)));

    let eof = got.last().expect("expected EOF token");
    assert_eq!(eof.0, TokenKind::Eof);
    assert_eq!(eof.2, Position::new(3, 1));
}

#[test]
fn string_literals_keep_raw_content() {
    let got: Vec<(TokenKind, String)> = collect("\"hello world\" \"a\\\\n b\"")
        .into_iter()
        .map(|(k, l, _)| (k, l))
        .collect();

    assert_eq!(
        got,
        vec![
            (TokenKind::String, "hello world".to_string()),
            (TokenKind::String, "a\\\\n b".to_string()),
            (TokenKind::Eof, "".to_string()),
        ]
    );
}

#[test]
fn unknown_char_emits_illegal() {
    let got: Vec<(TokenKind, String)> = collect("@").into_iter().map(|(k, l, _)| (k, l)).collect();
    assert_eq!(
        got,
        vec![
            (TokenKind::Illegal, "@".to_string()),
            (TokenKind::Eof, "".to_string())
        ]
    );
}

#[test]
fn unterminated_string_emits_illegal_then_eof() {
    let got: Vec<(TokenKind, String)> = collect("\"abc")
        .into_iter()
        .map(|(k, l, _)| (k, l))
        .collect();

    assert_eq!(
        got,
        vec![
            (TokenKind::Illegal, "abc".to_string()),
            (TokenKind::Eof, "".to_string()),
        ]
    );
}
