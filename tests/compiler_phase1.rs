use monkey_rust_compiler::ast::Program;
use monkey_rust_compiler::bytecode::{lookup_definition, read_operands, Chunk, Opcode};
use monkey_rust_compiler::compiler::{CompileError, Compiler};
use monkey_rust_compiler::lexer::Lexer;
use monkey_rust_compiler::object::Object;
use monkey_rust_compiler::parser::Parser;
use monkey_rust_compiler::position::Position;

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

fn compile_input(input: &str) -> Result<Chunk, CompileError> {
    let program = parse_program(input);
    let mut compiler = Compiler::new();
    compiler.compile_program(&program)?;
    Ok(compiler.into_bytecode())
}

fn compile_error(input: &str) -> CompileError {
    compile_input(input).expect_err("expected compile error")
}

fn decode_instructions(chunk: &Chunk) -> Vec<(usize, Opcode, Vec<usize>)> {
    let mut out = Vec::new();
    let mut offset = 0;

    while offset < chunk.instructions.len() {
        let op = Opcode::from_byte(chunk.instructions[offset])
            .unwrap_or_else(|| panic!("unknown opcode at offset {offset}"));
        let def = lookup_definition(op);
        let (operands, consumed) = read_operands(def, &chunk.instructions[offset + 1..])
            .unwrap_or_else(|err| panic!("failed decoding operands at {offset}: {err}"));
        out.push((offset, op, operands));
        offset += 1 + consumed;
    }

    out
}

#[test]
fn compiles_literals_and_expression_pop() {
    let cases = [
        (
            "1;",
            vec![(Opcode::Constant, vec![0]), (Opcode::Pop, vec![])],
            vec![Object::Integer(1)],
        ),
        (
            "true;",
            vec![(Opcode::True, vec![]), (Opcode::Pop, vec![])],
            vec![],
        ),
        (
            "false;",
            vec![(Opcode::False, vec![]), (Opcode::Pop, vec![])],
            vec![],
        ),
        (
            "\"abc\";",
            vec![(Opcode::Constant, vec![0]), (Opcode::Pop, vec![])],
            vec![Object::String("abc".to_string())],
        ),
    ];

    for (input, expected_ops, expected_constants) in cases {
        let chunk = compile_input(input).expect("compile should succeed");
        let decoded = decode_instructions(&chunk)
            .into_iter()
            .map(|(_, op, operands)| (op, operands))
            .collect::<Vec<_>>();
        assert_eq!(decoded, expected_ops, "input={input}");

        let constants = chunk
            .constants
            .iter()
            .map(|c| c.as_ref().clone())
            .collect::<Vec<_>>();
        assert_eq!(constants, expected_constants, "input={input}");
    }
}

#[test]
fn compiles_prefix_expressions() {
    let cases = [
        (
            "!true;",
            vec![
                (Opcode::True, vec![]),
                (Opcode::Bang, vec![]),
                (Opcode::Pop, vec![]),
            ],
            vec![],
        ),
        (
            "-5;",
            vec![
                (Opcode::Constant, vec![0]),
                (Opcode::Neg, vec![]),
                (Opcode::Pop, vec![]),
            ],
            vec![Object::Integer(5)],
        ),
        (
            "!(1 == 2);",
            vec![
                (Opcode::Constant, vec![0]),
                (Opcode::Constant, vec![1]),
                (Opcode::Eq, vec![]),
                (Opcode::Bang, vec![]),
                (Opcode::Pop, vec![]),
            ],
            vec![Object::Integer(1), Object::Integer(2)],
        ),
    ];

    for (input, expected_ops, expected_constants) in cases {
        let chunk = compile_input(input).expect("compile should succeed");
        let decoded = decode_instructions(&chunk)
            .into_iter()
            .map(|(_, op, operands)| (op, operands))
            .collect::<Vec<_>>();
        assert_eq!(decoded, expected_ops, "input={input}");

        let constants = chunk
            .constants
            .iter()
            .map(|c| c.as_ref().clone())
            .collect::<Vec<_>>();
        assert_eq!(constants, expected_constants, "input={input}");
    }
}

#[test]
fn compiles_infix_expressions() {
    let cases = [
        ("1 + 2;", Opcode::Add),
        ("1 - 2;", Opcode::Sub),
        ("1 * 2;", Opcode::Mul),
        ("4 / 2;", Opcode::Div),
        ("1 == 2;", Opcode::Eq),
        ("1 != 2;", Opcode::Ne),
        ("1 < 2;", Opcode::Lt),
        ("1 > 2;", Opcode::Gt),
        ("1 <= 2;", Opcode::Le),
        ("1 >= 2;", Opcode::Ge),
    ];

    for (input, expected_op) in cases {
        let chunk = compile_input(input).expect("compile should succeed");
        let decoded = decode_instructions(&chunk)
            .into_iter()
            .map(|(_, op, operands)| (op, operands))
            .collect::<Vec<_>>();

        assert_eq!(
            decoded,
            vec![
                (Opcode::Constant, vec![0]),
                (Opcode::Constant, vec![1]),
                (expected_op, vec![]),
                (Opcode::Pop, vec![]),
            ],
            "input={input}"
        );
    }
}

#[test]
fn grouped_precedence_uses_ast_shape() {
    let chunk = compile_input("(1 + 2) * 3;").expect("compile should succeed");
    let decoded = decode_instructions(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();

    assert_eq!(
        decoded,
        vec![
            (Opcode::Constant, vec![0]),
            (Opcode::Constant, vec![1]),
            (Opcode::Add, vec![]),
            (Opcode::Constant, vec![2]),
            (Opcode::Mul, vec![]),
            (Opcode::Pop, vec![]),
        ]
    );
}

#[test]
fn compiles_let_and_identifier_globals() {
    let chunk = compile_input("let a = 1; a;").expect("compile should succeed");
    let decoded = decode_instructions(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();

    assert_eq!(
        decoded,
        vec![
            (Opcode::Constant, vec![0]),
            (Opcode::SetGlobal, vec![0]),
            (Opcode::GetGlobal, vec![0]),
            (Opcode::Pop, vec![]),
        ]
    );

    let chunk = compile_input("let a = 1; let b = 2; a + b;").expect("compile should succeed");
    let decoded = decode_instructions(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();

    assert_eq!(
        decoded,
        vec![
            (Opcode::Constant, vec![0]),
            (Opcode::SetGlobal, vec![0]),
            (Opcode::Constant, vec![1]),
            (Opcode::SetGlobal, vec![1]),
            (Opcode::GetGlobal, vec![0]),
            (Opcode::GetGlobal, vec![1]),
            (Opcode::Add, vec![]),
            (Opcode::Pop, vec![]),
        ]
    );
}

#[test]
fn compiles_builtin_identifier_load() {
    let chunk = compile_input("len;").expect("compile should succeed");
    let decoded = decode_instructions(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();

    assert_eq!(
        decoded,
        vec![(Opcode::GetBuiltin, vec![0]), (Opcode::Pop, vec![])]
    );
}

#[test]
fn returns_deterministic_errors_for_unsupported_constructs() {
    let cases = [
        ("if (true) { 1 }", "unsupported expression in step 10: If"),
        (
            "fn(x) { x }",
            "unsupported expression in step 10: FunctionLiteral",
        ),
        ("[1,2,3]", "unsupported expression in step 10: ArrayLiteral"),
        (
            "{\"a\": 1}",
            "unsupported expression in step 10: HashLiteral",
        ),
        ("arr[0]", "unsupported expression in step 10: Index"),
        (
            "while (true) { }",
            "unsupported statement in step 10: While",
        ),
        ("return 1;", "unsupported statement in step 10: Return"),
        ("break;", "unsupported statement in step 10: Break"),
        ("continue;", "unsupported statement in step 10: Continue"),
        (
            "1 && 2;",
            "logical operator && compilation is not implemented in step 10",
        ),
        (
            "1 || 2;",
            "logical operator || compilation is not implemented in step 10",
        ),
    ];

    for (input, expected_message) in cases {
        let err = compile_error(input);
        assert_eq!(err.message, expected_message, "input={input}");
        assert!(err.pos.is_some(), "input={input}");
    }
}

#[test]
fn unresolved_identifier_is_compile_error() {
    let err = compile_error("foobar;");
    assert_eq!(err.message, "unresolved identifier: foobar");
    assert_eq!(err.pos, Some(Position::new(1, 1)));
}

#[test]
fn records_instruction_positions() {
    let chunk = compile_input("let a = 1;\na + 2;").expect("compile should succeed");
    let decoded = decode_instructions(&chunk);

    let first_constant_offset = decoded[0].0;
    let set_global_offset = decoded[1].0;
    let get_global_offset = decoded[2].0;
    let second_constant_offset = decoded[3].0;
    let add_offset = decoded[4].0;

    assert_eq!(
        chunk.position_for_offset(first_constant_offset),
        Some(Position::new(1, 9))
    );
    assert_eq!(
        chunk.position_for_offset(set_global_offset),
        Some(Position::new(1, 1))
    );
    assert_eq!(
        chunk.position_for_offset(get_global_offset),
        Some(Position::new(2, 1))
    );
    assert_eq!(
        chunk.position_for_offset(second_constant_offset),
        Some(Position::new(2, 5))
    );
    assert_eq!(
        chunk.position_for_offset(add_offset),
        Some(Position::new(2, 3))
    );
}
