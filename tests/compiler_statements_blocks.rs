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
fn return_statement_compiles_to_return_value() {
    let chunk = compile_input("return 1;").expect("compile should succeed");
    let decoded = decode_instructions(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert_eq!(
        decoded,
        vec![(Opcode::Constant, vec![0]), (Opcode::ReturnValue, vec![])]
    );

    let chunk = compile_input("return 1 + 2;").expect("compile should succeed");
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
            (Opcode::ReturnValue, vec![]),
        ]
    );
}

#[test]
fn top_level_final_expression_is_preserved() {
    let chunk = compile_input("1;").expect("compile should succeed");
    let decoded = decode_instructions(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert_eq!(
        decoded,
        vec![(Opcode::Constant, vec![0]), (Opcode::ReturnValue, vec![])]
    );

    let chunk = compile_input("1; 2;").expect("compile should succeed");
    let decoded = decode_instructions(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert_eq!(
        decoded,
        vec![
            (Opcode::Constant, vec![0]),
            (Opcode::Pop, vec![]),
            (Opcode::Constant, vec![1]),
            (Opcode::ReturnValue, vec![]),
        ]
    );

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
            (Opcode::ReturnValue, vec![]),
        ]
    );

    let chunk = compile_input("let a = 1;").expect("compile should succeed");
    let decoded = decode_instructions(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert_eq!(
        decoded,
        vec![
            (Opcode::Constant, vec![0]),
            (Opcode::SetGlobal, vec![0]),
            (Opcode::Return, vec![]),
        ]
    );
}

#[test]
fn logical_expression_regression_with_terminal_return_value() {
    let chunk = compile_input("true && false;").expect("compile should succeed");
    let ops = decode_instructions(&chunk)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();
    assert_eq!(ops.last(), Some(&Opcode::ReturnValue));
    assert!(ops.contains(&Opcode::JumpIfFalse));
    assert!(ops.contains(&Opcode::Jump));

    let chunk = compile_input("false || true;").expect("compile should succeed");
    let ops = decode_instructions(&chunk)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();
    assert_eq!(ops.last(), Some(&Opcode::ReturnValue));
    assert!(ops.contains(&Opcode::JumpIfFalse));
    assert!(ops.contains(&Opcode::Jump));
}

#[test]
fn unsupported_statement_errors_remain_deterministic() {
    let cases = [
        (
            "while (true) { }",
            "unsupported statement in step 12: While",
        ),
        ("break;", "unsupported statement in step 12: Break"),
        ("continue;", "unsupported statement in step 12: Continue"),
    ];

    for (input, expected) in cases {
        let err = compile_input(input).expect_err("expected compile error");
        assert_eq!(err.message, expected, "input={input}");
        assert!(err.pos.is_some(), "input={input}");
    }
}

#[test]
fn position_mapping_for_return_and_replaced_pop() {
    let chunk = compile_input("let a = 1;\na;\nreturn a;\n").expect("compile should succeed");
    let decoded = decode_instructions(&chunk);

    let set_global_offset = decoded
        .iter()
        .find(|(_, op, _)| *op == Opcode::SetGlobal)
        .map(|(offset, _, _)| *offset)
        .expect("expected SetGlobal");

    let get_global_offsets = decoded
        .iter()
        .filter(|(_, op, _)| *op == Opcode::GetGlobal)
        .map(|(offset, _, _)| *offset)
        .collect::<Vec<_>>();
    assert_eq!(get_global_offsets.len(), 2);

    let return_value_offset = decoded
        .iter()
        .find(|(_, op, _)| *op == Opcode::ReturnValue)
        .map(|(offset, _, _)| *offset)
        .expect("expected ReturnValue");

    assert_eq!(
        chunk.position_for_offset(set_global_offset),
        Some(Position::new(1, 1))
    );
    assert_eq!(
        chunk.position_for_offset(get_global_offsets[0]),
        Some(Position::new(2, 1))
    );
    assert_eq!(
        chunk.position_for_offset(get_global_offsets[1]),
        Some(Position::new(3, 8))
    );
    assert_eq!(
        chunk.position_for_offset(return_value_offset),
        Some(Position::new(3, 1))
    );

    let replaced_chunk = compile_input("let a = 1;\na;\n").expect("compile should succeed");
    let replaced_decoded = decode_instructions(&replaced_chunk);
    let replaced_return_offset = replaced_decoded
        .iter()
        .find(|(_, op, _)| *op == Opcode::ReturnValue)
        .map(|(offset, _, _)| *offset)
        .expect("expected replaced ReturnValue");
    assert_eq!(
        replaced_chunk.position_for_offset(replaced_return_offset),
        Some(Position::new(2, 1))
    );
}

#[test]
fn empty_program_emits_synthetic_return() {
    let chunk = compile_input("").expect("compile should succeed");
    let decoded = decode_instructions(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();

    assert_eq!(decoded, vec![(Opcode::Return, vec![])]);
    assert_eq!(chunk.position_for_offset(0), Some(Position::default()));
}
