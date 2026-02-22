use monkey_rust_compiler::position::Position;
use monkey_rust_compiler::runtime_error::{RuntimeError, RuntimeErrorType, StackFrameInfo};

#[test]
fn runtime_error_type_codes_are_protocol_stable() {
    let cases = [
        (RuntimeErrorType::TypeMismatch, "TYPE_MISMATCH"),
        (RuntimeErrorType::UnknownIdentifier, "UNKNOWN_IDENTIFIER"),
        (RuntimeErrorType::NotCallable, "NOT_CALLABLE"),
        (RuntimeErrorType::WrongArgumentCount, "WRONG_ARGUMENT_COUNT"),
        (
            RuntimeErrorType::InvalidArgumentType,
            "INVALID_ARGUMENT_TYPE",
        ),
        (RuntimeErrorType::InvalidControlFlow, "INVALID_CONTROL_FLOW"),
        (RuntimeErrorType::InvalidIndex, "INVALID_INDEX"),
        (RuntimeErrorType::Unhashable, "UNHASHABLE"),
        (RuntimeErrorType::DivisionByZero, "DIVISION_BY_ZERO"),
        (
            RuntimeErrorType::UnsupportedOperation,
            "UNSUPPORTED_OPERATION",
        ),
    ];

    for (error_type, expected_code) in cases {
        assert_eq!(error_type.code(), expected_code);
        assert_eq!(error_type.to_string(), expected_code);
    }
}

#[test]
fn runtime_error_construction_stores_fields_and_stack_helpers_work() {
    let mut err = RuntimeError::new(
        RuntimeErrorType::TypeMismatch,
        "unsupported operand types",
        Position::new(2, 7),
    );
    assert_eq!(err.error_type, RuntimeErrorType::TypeMismatch);
    assert_eq!(err.message, "unsupported operand types");
    assert_eq!(err.pos, Position::new(2, 7));
    assert!(err.stack.is_empty());

    err.push_frame(StackFrameInfo::new("add", Position::new(5, 3)).with_arg_count(2));
    assert_eq!(err.stack.len(), 1);

    let err = RuntimeError::at(
        RuntimeErrorType::UnknownIdentifier,
        Position::new(1, 1),
        "identifier not found: foobar",
    )
    .with_frame(StackFrameInfo::new("main", Position::new(9, 1)))
    .with_stack(vec![
        StackFrameInfo::new("add", Position::new(5, 3)).with_arg_count(2),
        StackFrameInfo::new("<repl>", Position::new(1, 1)).with_arg_count(0),
    ]);

    assert_eq!(err.error_type, RuntimeErrorType::UnknownIdentifier);
    assert_eq!(err.stack.len(), 2);
    assert_eq!(err.stack[0].function_name, "add");
    assert_eq!(err.stack[1].function_name, "<repl>");
}

#[test]
fn single_line_formatting_is_deterministic() {
    let type_mismatch = RuntimeError::new(
        RuntimeErrorType::TypeMismatch,
        "unsupported operand types: INTEGER + BOOLEAN",
        Position::new(2, 7),
    );
    assert_eq!(
        type_mismatch.format_single_line(),
        "Error[TYPE_MISMATCH] at 2:7: unsupported operand types: INTEGER + BOOLEAN"
    );
    assert_eq!(
        type_mismatch.to_string(),
        "Error[TYPE_MISMATCH] at 2:7: unsupported operand types: INTEGER + BOOLEAN"
    );

    let unknown = RuntimeError::new(
        RuntimeErrorType::UnknownIdentifier,
        "identifier not found: foobar",
        Position::new(1, 1),
    );
    assert_eq!(
        unknown.format_single_line(),
        "Error[UNKNOWN_IDENTIFIER] at 1:1: identifier not found: foobar"
    );

    let div_zero = RuntimeError::new(
        RuntimeErrorType::DivisionByZero,
        "division by zero",
        Position::new(4, 12),
    );
    assert_eq!(
        div_zero.format_single_line(),
        "Error[DIVISION_BY_ZERO] at 4:12: division by zero"
    );
}

#[test]
fn multiline_formatting_is_deterministic_with_and_without_stack() {
    let no_stack = RuntimeError::new(
        RuntimeErrorType::UnknownIdentifier,
        "identifier not found: foobar",
        Position::new(1, 1),
    );
    assert_eq!(
        no_stack.format_multiline(),
        "Error[UNKNOWN_IDENTIFIER] at 1:1: identifier not found: foobar"
    );

    let with_stack = RuntimeError::new(
        RuntimeErrorType::TypeMismatch,
        "unsupported operand types: INTEGER + BOOLEAN",
        Position::new(2, 7),
    )
    .with_stack(vec![
        StackFrameInfo::new("add", Position::new(5, 3)).with_arg_count(2),
        StackFrameInfo::new("main", Position::new(9, 1)).with_arg_count(1),
        StackFrameInfo::new("<repl>", Position::new(1, 1)).with_arg_count(0),
    ]);

    let expected = "Error[TYPE_MISMATCH] at 2:7: unsupported operand types: INTEGER + BOOLEAN\nStack trace:\n  at add(2 args) @ 5:3\n  at main(1 args) @ 9:1\n  at <repl>(0 args) @ 1:1";
    assert_eq!(with_stack.format_multiline(), expected);
}

#[test]
fn stack_frame_formatting_is_stable() {
    let with_args = StackFrameInfo::new("myFunc", Position::new(3, 14)).with_arg_count(3);
    assert_eq!(with_args.format_frame(), "at myFunc(3 args) @ 3:14");

    let without_args = StackFrameInfo::new("<repl>", Position::new(1, 1));
    assert_eq!(without_args.format_frame(), "at <repl> @ 1:1");
}

#[test]
fn position_propagates_in_all_formatted_outputs() {
    let err = RuntimeError::new(
        RuntimeErrorType::InvalidControlFlow,
        "break used outside loop",
        Position::new(12, 4),
    )
    .with_frame(StackFrameInfo::new("<repl>", Position::new(1, 1)).with_arg_count(0));

    assert!(err.format_single_line().contains("12:4"));
    assert!(err.format_multiline().contains("12:4"));
    assert!(err.format_multiline().contains("1:1"));
}
