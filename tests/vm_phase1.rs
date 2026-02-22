use monkey_rust_compiler::ast::Program;
use monkey_rust_compiler::compiler::{CompileError, Compiler};
use monkey_rust_compiler::lexer::Lexer;
use monkey_rust_compiler::object::Object;
use monkey_rust_compiler::parser::Parser;
use monkey_rust_compiler::position::Position;
use monkey_rust_compiler::runtime_error::{RuntimeError, RuntimeErrorType};
use monkey_rust_compiler::vm::Vm;

fn parse_program(input: &str) -> Program {
    let mut parser = Parser::new(Lexer::new(input));
    let program = parser.parse_program();
    let errors = parser.errors();
    assert!(
        errors.is_empty(),
        "expected no parse errors for input:\n{input}\nerrors:\n{}",
        errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
    program
}

fn run_input(input: &str) -> Result<Object, RuntimeError> {
    let mut compiler = Compiler::new();
    compiler
        .compile_program(&parse_program(input))
        .map_err(|err: CompileError| {
            panic!(
                "compile failed for input:\n{input}\nmessage: {}\npos: {:?}",
                err.message, err.pos
            )
        })?;
    let chunk = compiler.into_bytecode();
    let mut vm = Vm::new(chunk);
    vm.run().map(|obj| obj.as_ref().clone())
}

fn assert_int(obj: Object, expected: i64) {
    assert_eq!(obj, Object::Integer(expected));
}

fn assert_bool(obj: Object, expected: bool) {
    assert_eq!(obj, Object::Boolean(expected));
}

fn assert_string(obj: Object, expected: &str) {
    assert_eq!(obj, Object::String(expected.to_string()));
}

fn assert_null(obj: Object) {
    assert_eq!(obj, Object::Null);
}

#[test]
fn executes_literals() {
    assert_int(run_input("1;").expect("vm run should succeed"), 1);
    assert_bool(run_input("true;").expect("vm run should succeed"), true);
    assert_bool(run_input("false;").expect("vm run should succeed"), false);
    assert_string(run_input("\"abc\";").expect("vm run should succeed"), "abc");
    assert_null(run_input("if (false) { 1 };").expect("vm run should succeed"));
}

#[test]
fn executes_arithmetic_and_prefix() {
    assert_int(run_input("1 + 2;").expect("vm run should succeed"), 3);
    assert_int(run_input("5 - 3;").expect("vm run should succeed"), 2);
    assert_int(run_input("2 * 3;").expect("vm run should succeed"), 6);
    assert_int(run_input("8 / 2;").expect("vm run should succeed"), 4);
    assert_int(run_input("-5;").expect("vm run should succeed"), -5);
    assert_int(run_input("(1 + 2) * 3;").expect("vm run should succeed"), 9);
    assert_string(
        run_input("\"a\" + \"b\";").expect("vm run should succeed"),
        "ab",
    );
}

#[test]
fn executes_comparisons_and_bang_truthiness() {
    assert_bool(run_input("1 == 1;").expect("vm run should succeed"), true);
    assert_bool(run_input("1 != 2;").expect("vm run should succeed"), true);
    assert_bool(run_input("1 < 2;").expect("vm run should succeed"), true);
    assert_bool(run_input("2 <= 2;").expect("vm run should succeed"), true);
    assert_bool(run_input("3 >= 4;").expect("vm run should succeed"), false);
    assert_bool(
        run_input("true == false;").expect("vm run should succeed"),
        false,
    );
    assert_bool(
        run_input("if (false) { 1 } == if (false) { 2 };").expect("vm run should succeed"),
        true,
    );

    assert_bool(run_input("!true;").expect("vm run should succeed"), false);
    assert_bool(run_input("!false;").expect("vm run should succeed"), true);
    assert_bool(
        run_input("!(if (false) { 1 });").expect("vm run should succeed"),
        true,
    );
    assert_bool(run_input("!5;").expect("vm run should succeed"), false);
    assert_bool(run_input("!!5;").expect("vm run should succeed"), true);
}

#[test]
fn executes_short_circuit_logic_with_boolean_results() {
    assert_bool(
        run_input("true && false;").expect("vm run should succeed"),
        false,
    );
    assert_bool(
        run_input("true && 123;").expect("vm run should succeed"),
        true,
    );
    assert_bool(
        run_input("false && 123;").expect("vm run should succeed"),
        false,
    );
    assert_bool(
        run_input("false || 123;").expect("vm run should succeed"),
        true,
    );
    assert_bool(
        run_input("true || 123;").expect("vm run should succeed"),
        true,
    );
    assert_bool(
        run_input("(1 < 2) && (2 < 3);").expect("vm run should succeed"),
        true,
    );
}

#[test]
fn executes_globals_if_while_and_return_semantics() {
    assert_int(
        run_input("let a = 1; a;").expect("vm run should succeed"),
        1,
    );
    assert_int(
        run_input("let a = 1; let b = 2; a + b;").expect("vm run should succeed"),
        3,
    );
    assert_int(
        run_input("let a = 1; let b = a + 2; b;").expect("vm run should succeed"),
        3,
    );

    assert_int(
        run_input("if (true) { 10 } else { 20 };").expect("vm run should succeed"),
        10,
    );
    assert_int(
        run_input("if (false) { 10 } else { 20 };").expect("vm run should succeed"),
        20,
    );
    assert_null(run_input("if (false) { 10 };").expect("vm run should succeed"));
    assert_int(
        run_input("if (1) { 10 } else { 20 };").expect("vm run should succeed"),
        10,
    );

    assert_null(run_input("while (false) { 1; }").expect("vm run should succeed"));
    assert_null(run_input("while (true) { break; }").expect("vm run should succeed"));
    assert_null(run_input("while (true) { break; continue; }").expect("vm run should succeed"));

    assert_int(run_input("return 1;").expect("vm run should succeed"), 1);
    assert_int(
        run_input("if (true) { return 5; } 10;").expect("vm run should succeed"),
        5,
    );
    assert_int(
        run_input("while (true) { return 7; }").expect("vm run should succeed"),
        7,
    );
}

#[test]
fn invalid_top_level_break_continue_are_runtime_errors() {
    let err = run_input("break;").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::InvalidControlFlow);
    assert_eq!(err.message, "break used outside of loop");
    assert_eq!(err.pos, Position::new(1, 1));

    let err = run_input("continue;").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::InvalidControlFlow);
    assert_eq!(err.message, "continue used outside of loop");
    assert_eq!(err.pos, Position::new(1, 1));
}

#[test]
fn supported_runtime_errors_are_deterministic() {
    let err = run_input("1 / 0;").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::DivisionByZero);
    assert_eq!(err.message, "division by zero");
    assert_eq!(err.pos, Position::new(1, 3));

    let err = run_input("-true;").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::TypeMismatch);
    assert_eq!(err.message, "unsupported operand type for -: BOOLEAN");

    let err = run_input("1 + true;").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::TypeMismatch);
    assert_eq!(
        err.message,
        "unsupported operand types for Add: INTEGER and BOOLEAN"
    );

    let err = run_input("1 < true;").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::TypeMismatch);
    assert_eq!(
        err.message,
        "unsupported operand types for Lt: INTEGER and BOOLEAN"
    );
}

#[test]
fn deferred_opcode_boundary_errors_are_deterministic() {
    let err = run_input("fn() { 1; };").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::UnsupportedOperation);
    assert_eq!(err.message, "opcode not implemented in step 16: Closure");

    let err = run_input("len(\"abc\");").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::UnsupportedOperation);
    assert_eq!(err.message, "opcode not implemented in step 16: GetBuiltin");

    let err = run_input("[1, 2];").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::UnsupportedOperation);
    assert_eq!(err.message, "opcode not implemented in step 16: Array");

    let err = run_input("{\"a\": 1};").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::UnsupportedOperation);
    assert_eq!(err.message, "opcode not implemented in step 16: Hash");

    let err = run_input("let a = 1; let i = 0; a[i];").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::UnsupportedOperation);
    assert_eq!(err.message, "opcode not implemented in step 16: Index");
}

#[test]
fn runtime_error_position_propagates_from_chunk_metadata() {
    let err = run_input("let a = 1;\nlet b = 0;\na / b;\n").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::DivisionByZero);
    assert_eq!(err.pos, Position::new(3, 3));
}
