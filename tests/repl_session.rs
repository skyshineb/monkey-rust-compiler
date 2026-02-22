use monkey_rust_compiler::repl::{ReplEvalResult, ReplSession};

#[test]
fn state_persists_across_lines() {
    let mut repl = ReplSession::new();

    match repl.eval_line("let a = 10;") {
        ReplEvalResult::Value { .. } => {}
        other => panic!("expected value result, got {other:?}"),
    }

    match repl.eval_line("a + 5;") {
        ReplEvalResult::Value { result, .. } => assert_eq!(result.inspect(), "15"),
        other => panic!("expected value result, got {other:?}"),
    }
}

#[test]
fn closures_persist_across_lines() {
    let mut repl = ReplSession::new();
    match repl.eval_line("let newAdder = fn(a) { fn(b) { a + b } };") {
        ReplEvalResult::Value { .. } => {}
        other => panic!("expected value result, got {other:?}"),
    }
    match repl.eval_line("let addTwo = newAdder(2);") {
        ReplEvalResult::Value { .. } => {}
        other => panic!("expected value result, got {other:?}"),
    }
    match repl.eval_line("addTwo(3);") {
        ReplEvalResult::Value { result, .. } => assert_eq!(result.inspect(), "5"),
        other => panic!("expected value result, got {other:?}"),
    }
}

#[test]
fn repl_handles_errors_deterministically() {
    let mut repl = ReplSession::new();

    match repl.eval_line("let = ;") {
        ReplEvalResult::ParseErrors(errors) => assert!(!errors.is_empty()),
        other => panic!("expected parse errors, got {other:?}"),
    }

    match repl.eval_line("1 / 0;") {
        ReplEvalResult::RuntimeError(err) => {
            assert_eq!(err.error_type.code(), "DIVISION_BY_ZERO");
            assert_eq!(err.pos.line, 1);
        }
        other => panic!("expected runtime error, got {other:?}"),
    }
}

#[test]
fn repl_multiline_buffering_and_meta_gating() {
    let mut repl = ReplSession::new();

    match repl.eval_line("let add = fn(a, b) {") {
        ReplEvalResult::Empty => {}
        other => panic!("expected buffered empty, got {other:?}"),
    }

    match repl.eval_line(":help") {
        ReplEvalResult::Empty => {}
        other => panic!("expected buffered empty for meta text, got {other:?}"),
    }

    match repl.eval_line("a + b") {
        ReplEvalResult::Empty => {}
        other => panic!("expected buffered empty, got {other:?}"),
    }

    match repl.eval_line("};") {
        ReplEvalResult::ParseErrors(errors) => assert!(!errors.is_empty()),
        other => panic!("expected parse error from buffered :help line, got {other:?}"),
    }

    match repl.eval_line(":help") {
        ReplEvalResult::MetaOutput(text) => assert!(text.contains("Commands:")),
        other => panic!("expected meta output after buffer reset, got {other:?}"),
    }
}

#[test]
fn meta_commands_work() {
    let mut repl = ReplSession::new();

    match repl.eval_line(":help") {
        ReplEvalResult::MetaOutput(text) => assert!(text.contains(":tokens")),
        other => panic!("expected meta output, got {other:?}"),
    }

    match repl.eval_line(":tokens let a = 1;") {
        ReplEvalResult::MetaOutput(text) => assert!(text.contains("TOKENS:")),
        other => panic!("expected meta output, got {other:?}"),
    }

    match repl.eval_line(":ast 1 + 2;") {
        ReplEvalResult::MetaOutput(text) => assert!(text.contains("AST:")),
        other => panic!("expected meta output, got {other:?}"),
    }

    match repl.eval_line(":env") {
        ReplEvalResult::MetaOutput(text) => assert!(text.starts_with("ENV:")),
        other => panic!("expected meta output, got {other:?}"),
    }

    match repl.eval_line(":quit") {
        ReplEvalResult::ExitRequested => {}
        other => panic!("expected exit request, got {other:?}"),
    }
}
