use std::fs;

use monkey_rust_compiler::lexer::Lexer;
use monkey_rust_compiler::parser::Parser;

#[test]
fn benchmark_sources_parse_without_errors() {
    for name in ["b1", "b2", "b3", "b4", "b5"] {
        let path = format!("bench/{name}.monkey");
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read benchmark source {path}: {e}"));
        let mut parser = Parser::new(Lexer::new(&source));
        let _ = parser.parse_program();
        let errors = parser.errors();
        assert!(
            errors.is_empty(),
            "expected no parse errors in {path}, got:\n{}",
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
