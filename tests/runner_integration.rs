use monkey_rust_compiler::runner::{dump_ast, format_tokens, run_source, RunnerError};

#[test]
fn dump_tokens_is_deterministic() {
    let out = format_tokens("let a = 1;");
    let expected = [
        "Let('let') @ 1:1",
        "Ident('a') @ 1:5",
        "Assign('=') @ 1:7",
        "Int('1') @ 1:9",
        "Semicolon(';') @ 1:10",
        "Eof('') @ 1:11",
    ]
    .join("\n");
    assert_eq!(out, expected);
}

#[test]
fn dump_ast_is_deterministic() {
    let ast = dump_ast("1 + 2 * 3;").expect("ast should parse");
    assert_eq!(ast, "(1 + (2 * 3));");

    let ast = dump_ast("let add = fn(a, b) { a + b }; add(1, 2);").expect("ast should parse");
    assert_eq!(ast, "let add = fn(a, b) { (a + b); };\nadd(1, 2);");
}

#[test]
fn run_source_executes_pipeline() {
    let out = run_source("1 + 2;").expect("run should succeed");
    assert_eq!(out.result.inspect(), "3");
    assert!(out.output.is_empty());

    let out = run_source("puts(\"x\"); 1;").expect("run should succeed");
    assert_eq!(out.result.inspect(), "1");
    assert_eq!(out.output, vec!["x".to_string()]);
}

#[test]
fn run_source_error_shapes_are_deterministic() {
    match run_source("let = ;") {
        Err(RunnerError::Parse(errors)) => assert!(!errors.is_empty()),
        other => panic!("expected parse error, got {other:?}"),
    }

    match run_source("1 / 0;") {
        Err(RunnerError::Runtime(err)) => {
            assert_eq!(err.error_type.code(), "DIVISION_BY_ZERO");
            assert_eq!(err.pos.line, 1);
        }
        other => panic!("expected runtime error, got {other:?}"),
    }
}
