use monkey_rust_compiler::ast::Program;
use monkey_rust_compiler::compiler::{CompileError, Compiler};
use monkey_rust_compiler::lexer::Lexer;
use monkey_rust_compiler::object::Object;
use monkey_rust_compiler::parser::Parser;
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

fn compile_to_vm(input: &str) -> Vm {
    let mut compiler = Compiler::new();
    compiler
        .compile_program(&parse_program(input))
        .map_err(|err: CompileError| {
            panic!(
                "compile failed for input:\n{input}\nmessage: {}\npos: {:?}",
                err.message, err.pos
            )
        })
        .expect("compilation should succeed");
    Vm::new(compiler.into_bytecode())
}

fn run_input(input: &str) -> Result<Object, RuntimeError> {
    let mut vm = compile_to_vm(input);
    vm.run().map(|obj| obj.as_ref().clone())
}

#[test]
fn executes_function_calls_and_locals() {
    assert_eq!(
        run_input("let id = fn(x) { x }; id(5);").expect("vm run should succeed"),
        Object::Integer(5)
    );
    assert_eq!(
        run_input("let add = fn(a, b) { a + b }; add(2, 3);").expect("vm run should succeed"),
        Object::Integer(5)
    );
    assert_eq!(
        run_input("fn(x) { x + 1 }(4);").expect("vm run should succeed"),
        Object::Integer(5)
    );

    let src =
        "let add = fn(a, b) { a + b }; let apply = fn(x, y, f) { f(x, y) }; apply(2, 3, add);";
    assert_eq!(
        run_input(src).expect("vm run should succeed"),
        Object::Integer(5)
    );

    let src = "let f = fn(a) { let b = a + 1; b + 2 }; f(3);";
    assert_eq!(
        run_input(src).expect("vm run should succeed"),
        Object::Integer(6)
    );
}

#[test]
fn executes_closures_and_recursion() {
    let src = r#"
let newAdder = fn(a) {
  fn(b) { a + b }
};
let addTwo = newAdder(2);
addTwo(5);
"#;
    assert_eq!(
        run_input(src).expect("vm run should succeed"),
        Object::Integer(7)
    );

    let src = r#"
let outer = fn(a) {
  fn(b) {
    fn(c) { a + b + c }
  }
};
outer(1)(2)(3);
"#;
    assert_eq!(
        run_input(src).expect("vm run should succeed"),
        Object::Integer(6)
    );

    let src = r#"
let fact = fn(n) {
  if (n == 0) { 1 } else { n * fact(n - 1) }
};
fact(5);
"#;
    assert_eq!(
        run_input(src).expect("vm run should succeed"),
        Object::Integer(120)
    );
}

#[test]
fn executes_builtins_and_captures_puts_output() {
    assert_eq!(
        run_input("len(\"\");").expect("vm run should succeed"),
        Object::Integer(0)
    );
    assert_eq!(
        run_input("len(\"abc\");").expect("vm run should succeed"),
        Object::Integer(3)
    );
    assert_eq!(
        run_input("len([1,2,3]);").expect("vm run should succeed"),
        Object::Integer(3)
    );

    assert_eq!(
        run_input("first([1,2,3]);").expect("vm run should succeed"),
        Object::Integer(1)
    );
    assert_eq!(
        run_input("last([1,2,3]);").expect("vm run should succeed"),
        Object::Integer(3)
    );
    assert_eq!(
        run_input("first([]);").expect("vm run should succeed"),
        Object::Null
    );
    assert_eq!(
        run_input("last([]);").expect("vm run should succeed"),
        Object::Null
    );
    assert_eq!(
        run_input("rest([]);").expect("vm run should succeed"),
        Object::Null
    );
    assert_eq!(
        run_input("rest([1,2,3]);").expect("vm run should succeed"),
        Object::Array(vec![Object::Integer(2).rc(), Object::Integer(3).rc()])
    );
    assert_eq!(
        run_input("push([1,2], 3);").expect("vm run should succeed"),
        Object::Array(vec![
            Object::Integer(1).rc(),
            Object::Integer(2).rc(),
            Object::Integer(3).rc()
        ])
    );

    let mut vm = compile_to_vm("puts(\"a\", 1, true);");
    let result = vm.run().expect("vm run should succeed");
    assert_eq!(result.as_ref(), &Object::Null);
    assert_eq!(vm.take_output(), vec!["a1true".to_string()]);
}

#[test]
fn builtin_errors_are_deterministic() {
    let err = run_input("len(1);").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::InvalidArgumentType);
    assert_eq!(err.message, "len expected STRING or ARRAY, got INTEGER");

    let err = run_input("len(\"a\", \"b\");").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::WrongArgumentCount);
    assert_eq!(err.message, "len expected 1 argument(s), got 2");

    let err = run_input("first(1);").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::InvalidArgumentType);
    assert_eq!(err.message, "first expected ARRAY, got INTEGER");
}

#[test]
fn executes_arrays_hashes_and_indexing() {
    assert_eq!(
        run_input("[];").expect("vm run should succeed"),
        Object::Array(vec![])
    );
    assert_eq!(
        run_input("[1, 2, 3];").expect("vm run should succeed"),
        Object::Array(vec![
            Object::Integer(1).rc(),
            Object::Integer(2).rc(),
            Object::Integer(3).rc()
        ])
    );
    assert_eq!(
        run_input("[1 + 2, 3 * 4];").expect("vm run should succeed"),
        Object::Array(vec![Object::Integer(3).rc(), Object::Integer(12).rc()])
    );

    assert_eq!(
        run_input("{};").expect("vm run should succeed"),
        Object::Hash(vec![])
    );
    assert_eq!(
        run_input("{\"a\": 1, \"b\": 2};").expect("vm run should succeed"),
        Object::Hash(vec![
            (
                Object::String("a".to_string()).rc(),
                Object::Integer(1).rc()
            ),
            (
                Object::String("b".to_string()).rc(),
                Object::Integer(2).rc()
            )
        ])
    );
    assert_eq!(
        run_input("{\"a\": 1, \"a\": 2}[\"a\"];").expect("vm run should succeed"),
        Object::Integer(2)
    );

    assert_eq!(
        run_input("[1,2,3][0];").expect("vm run should succeed"),
        Object::Integer(1)
    );
    assert_eq!(
        run_input("[1,2,3][2];").expect("vm run should succeed"),
        Object::Integer(3)
    );
    assert_eq!(
        run_input("[1,2,3][3];").expect("vm run should succeed"),
        Object::Null
    );
    assert_eq!(
        run_input("[1,2,3][-1];").expect("vm run should succeed"),
        Object::Null
    );
    assert_eq!(
        run_input("{\"a\": 1}[\"a\"];").expect("vm run should succeed"),
        Object::Integer(1)
    );
    assert_eq!(
        run_input("{\"a\": 1}[\"b\"];").expect("vm run should succeed"),
        Object::Null
    );
}

#[test]
fn indexing_errors_are_deterministic() {
    let err = run_input("1[0];").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::InvalidIndex);
    assert_eq!(err.message, "index operator not supported: INTEGER");

    let err = run_input("[1,2][\"x\"];").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::InvalidIndex);
    assert_eq!(err.message, "array index must be INTEGER, got STRING");

    let err = run_input("{\"a\": 1}[[]];").expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::Unhashable);
    assert_eq!(err.message, "unusable as hash key: ARRAY");
}

#[test]
fn full_feature_integration_works() {
    let src = r#"
let mk = fn(a) {
  fn(b) {
    let arr = [a, b, a + b];
    {"sum": arr[2], "len": len(arr)}
  }
};
let f = mk(2);
let h = f(5);
h["sum"] + h["len"];
"#;
    assert_eq!(
        run_input(src).expect("vm run should succeed"),
        Object::Integer(10)
    );
}

#[test]
fn runtime_error_stack_traces_are_populated() {
    let src = r#"
let bad = fn(x) { x + true };
let mid = fn(y) { bad(y) };
mid(1);
"#;
    let err = run_input(src).expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::TypeMismatch);
    assert!(err.stack.len() >= 2);
    assert_eq!(err.stack[0].function_name, "bad");
    assert_eq!(err.stack[1].function_name, "mid");
    assert!(err.stack.iter().all(|f| f.arg_count.is_some()));

    let src = r#"
let f = fn() { len(1) };
f();
"#;
    let err = run_input(src).expect_err("expected runtime error");
    assert_eq!(err.error_type, RuntimeErrorType::InvalidArgumentType);
    assert!(!err.stack.is_empty());
    assert_eq!(err.stack[0].function_name, "f");
}
