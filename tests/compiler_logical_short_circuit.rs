use monkey_rust_compiler::ast::Program;
use monkey_rust_compiler::bytecode::{lookup_definition, read_operands, Chunk, Opcode};
use monkey_rust_compiler::compiler::{CompileError, Compiler};
use monkey_rust_compiler::lexer::Lexer;
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
    let mut compiler = Compiler::new();
    compiler.compile_program(&parse_program(input))?;
    Ok(compiler.into_bytecode())
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
fn compiles_and_with_short_circuit_shape() {
    let chunk = compile_input("true && false;").expect("compile should succeed");
    let decoded = decode_instructions(&chunk);
    let simple = decoded
        .iter()
        .map(|(_, op, operands)| (*op, operands.clone()))
        .collect::<Vec<_>>();

    assert_eq!(simple[0], (Opcode::True, vec![]));
    assert_eq!(simple[1].0, Opcode::JumpIfFalse);
    assert_eq!(simple[2], (Opcode::Pop, vec![]));
    assert_eq!(simple[3], (Opcode::False, vec![]));
    assert_eq!(simple[4], (Opcode::Bang, vec![]));
    assert_eq!(simple[5], (Opcode::Bang, vec![]));
    assert_eq!(simple[6].0, Opcode::Jump);
    assert_eq!(simple[7], (Opcode::Pop, vec![]));
    assert_eq!(simple[8], (Opcode::False, vec![]));
    assert_eq!(simple[9], (Opcode::Pop, vec![]));

    let jump_if_false_target = simple[1].1[0];
    let jump_end_target = simple[6].1[0];
    assert_eq!(jump_if_false_target, decoded[7].0);
    assert_eq!(jump_end_target, decoded[9].0);
}

#[test]
fn compiles_or_with_short_circuit_shape() {
    let chunk = compile_input("false || true;").expect("compile should succeed");
    let decoded = decode_instructions(&chunk);
    let simple = decoded
        .iter()
        .map(|(_, op, operands)| (*op, operands.clone()))
        .collect::<Vec<_>>();

    assert_eq!(simple[0], (Opcode::False, vec![]));
    assert_eq!(simple[1].0, Opcode::JumpIfFalse);
    assert_eq!(simple[2], (Opcode::Pop, vec![]));
    assert_eq!(simple[3], (Opcode::True, vec![]));
    assert_eq!(simple[4].0, Opcode::Jump);
    assert_eq!(simple[5], (Opcode::Pop, vec![]));
    assert_eq!(simple[6], (Opcode::True, vec![]));
    assert_eq!(simple[7], (Opcode::Bang, vec![]));
    assert_eq!(simple[8], (Opcode::Bang, vec![]));
    assert_eq!(simple[9], (Opcode::Pop, vec![]));

    let jump_if_false_target = simple[1].1[0];
    let jump_end_target = simple[4].1[0];
    assert_eq!(jump_if_false_target, decoded[5].0);
    assert_eq!(jump_end_target, decoded[9].0);
}

#[test]
fn normalizes_rhs_to_boolean_for_non_boolean_operands() {
    let and_chunk = compile_input("1 && 2;").expect("compile should succeed");
    let and_ops = decode_instructions(&and_chunk)
        .iter()
        .map(|(_, op, _)| *op)
        .collect::<Vec<_>>();
    assert!(and_ops
        .windows(2)
        .any(|w| w == [Opcode::Bang, Opcode::Bang]));
    assert!(and_ops.contains(&Opcode::False));

    let or_chunk = compile_input("0 || \"x\";").expect("compile should succeed");
    let or_ops = decode_instructions(&or_chunk)
        .iter()
        .map(|(_, op, _)| *op)
        .collect::<Vec<_>>();
    assert!(or_ops.windows(2).any(|w| w == [Opcode::Bang, Opcode::Bang]));
    assert!(or_ops.contains(&Opcode::True));
}

#[test]
fn nested_logical_precedence_is_reflected_in_bytecode() {
    for input in [
        "let a = true; let b = false; let c = true; a && b || c;",
        "let a = true; let b = false; let c = true; a || b && c;",
    ] {
        let chunk = compile_input(input).expect("compile should succeed");
        let ops = decode_instructions(&chunk)
            .iter()
            .map(|(_, op, _)| *op)
            .collect::<Vec<_>>();

        let jump_if_false_count = ops.iter().filter(|&&op| op == Opcode::JumpIfFalse).count();
        let jump_count = ops.iter().filter(|&&op| op == Opcode::Jump).count();
        assert!(jump_if_false_count >= 2, "input={input}");
        assert!(jump_count >= 2, "input={input}");
    }
}

#[test]
fn grouped_logical_expression_respects_parentheses() {
    let chunk = compile_input("let a = true; let b = false; let c = true; (a || b) && c;")
        .expect("compile should succeed");
    let ops = decode_instructions(&chunk)
        .iter()
        .map(|(_, op, _)| *op)
        .collect::<Vec<_>>();

    let first_true = ops
        .iter()
        .position(|op| *op == Opcode::True)
        .expect("expected synthetic True from || short-circuit branch");
    let first_false = ops
        .iter()
        .position(|op| *op == Opcode::False)
        .expect("expected synthetic False from && short-circuit branch");
    assert!(first_true < first_false);
}

#[test]
fn let_rhs_logical_expression_keeps_value_for_setglobal() {
    for input in [
        "let a = true; let b = false; let x = a && b;",
        "let a = true; let b = false; let y = a || b;",
    ] {
        let chunk = compile_input(input).expect("compile should succeed");
        let decoded = decode_instructions(&chunk);

        let set_global_idx = decoded
            .iter()
            .position(|(_, op, _)| *op == Opcode::SetGlobal)
            .expect("expected SetGlobal");
        assert!(set_global_idx > 0, "input={input}");

        let before_set = decoded[set_global_idx - 1].1;
        assert_ne!(before_set, Opcode::Pop, "input={input}");
    }
}

#[test]
fn records_positions_for_jump_and_synthetic_logical_ops() {
    let input = "let a = true;\nlet b = false;\na && b;\na || b;\n";
    let chunk = compile_input(input).expect("compile should succeed");
    let decoded = decode_instructions(&chunk);

    let mut saw_and_jump = false;
    let mut saw_or_jump = false;
    let mut saw_and_false = false;
    let mut saw_or_true = false;
    let mut saw_and_pop = false;
    let mut saw_or_pop = false;

    for (offset, op, _) in decoded {
        let pos = chunk
            .position_for_offset(offset)
            .expect("expected position metadata for emitted instruction");

        if op == Opcode::JumpIfFalse && pos == Position::new(3, 3) {
            saw_and_jump = true;
        }
        if op == Opcode::JumpIfFalse && pos == Position::new(4, 3) {
            saw_or_jump = true;
        }
        if op == Opcode::False && pos == Position::new(3, 3) {
            saw_and_false = true;
        }
        if op == Opcode::True && pos == Position::new(4, 3) {
            saw_or_true = true;
        }
        if op == Opcode::Pop && pos == Position::new(3, 3) {
            saw_and_pop = true;
        }
        if op == Opcode::Pop && pos == Position::new(4, 3) {
            saw_or_pop = true;
        }
    }

    assert!(saw_and_jump);
    assert!(saw_or_jump);
    assert!(saw_and_false);
    assert!(saw_or_true);
    assert!(saw_and_pop);
    assert!(saw_or_pop);
}

#[test]
fn non_logical_infix_regression_unchanged() {
    for input in ["1 + 2;", "1 == 2;"] {
        let chunk = compile_input(input).expect("compile should succeed");
        let ops = decode_instructions(&chunk)
            .iter()
            .map(|(_, op, _)| *op)
            .collect::<Vec<_>>();

        assert!(!ops.contains(&Opcode::JumpIfFalse), "input={input}");
        assert!(!ops.contains(&Opcode::Jump), "input={input}");
    }
}

#[test]
fn unsupported_constructs_still_error() {
    let cases = [
        ("if (true) { 1 }", "unsupported expression in step 10: If"),
        (
            "fn(x) { x }",
            "unsupported expression in step 10: FunctionLiteral",
        ),
        ("[1, 2]", "unsupported expression in step 10: ArrayLiteral"),
    ];

    for (input, expected) in cases {
        let err = compile_input(input).expect_err("expected compile error");
        assert_eq!(err.message, expected, "input={input}");
    }
}
