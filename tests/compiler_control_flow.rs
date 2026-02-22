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
fn if_without_else_compiles_and_pushes_null_on_false() {
    let chunk = compile_input("if (true) { 10; };").expect("compile should succeed");
    let decoded = decode_instructions(&chunk);
    let ops = decoded.iter().map(|(_, op, _)| *op).collect::<Vec<_>>();

    assert!(ops.starts_with(&[Opcode::True, Opcode::JumpIfFalse, Opcode::Pop]));
    assert!(ops.contains(&Opcode::Null));
    assert_eq!(ops.last(), Some(&Opcode::ReturnValue));

    let jump_if_false = decoded
        .iter()
        .find(|(_, op, _)| *op == Opcode::JumpIfFalse)
        .expect("expected JumpIfFalse");
    let jump_end = decoded
        .iter()
        .find(|(_, op, _)| *op == Opcode::Jump)
        .expect("expected Jump");

    assert!(jump_if_false.2[0] > jump_if_false.0);
    assert!(jump_end.2[0] > jump_end.0);
}

#[test]
fn if_else_compiles_with_value_branches() {
    let chunk = compile_input("if (true) { 10; } else { 20; };").expect("compile should succeed");
    let decoded = decode_instructions(&chunk);
    let ops = decoded.iter().map(|(_, op, _)| *op).collect::<Vec<_>>();

    assert!(ops.contains(&Opcode::JumpIfFalse));
    assert!(ops.iter().filter(|&&op| op == Opcode::Jump).count() >= 1);
    assert_eq!(ops.last(), Some(&Opcode::ReturnValue));

    let const_count = ops.iter().filter(|&&op| op == Opcode::Constant).count();
    assert_eq!(const_count, 2);
}

#[test]
fn if_branch_fallback_to_null_when_no_value() {
    let chunk = compile_input("if (true) { let a = 1; };").expect("compile should succeed");
    let ops = decode_instructions(&chunk)
        .iter()
        .map(|(_, op, _)| *op)
        .collect::<Vec<_>>();
    assert!(ops.contains(&Opcode::Null));

    let chunk = compile_input("if (false) { };").expect("compile should succeed");
    let ops = decode_instructions(&chunk)
        .iter()
        .map(|(_, op, _)| *op)
        .collect::<Vec<_>>();
    let null_count = ops.iter().filter(|&&op| op == Opcode::Null).count();
    assert!(null_count >= 1);
}

#[test]
fn nested_if_compiles_deterministically() {
    let chunk = compile_input(
        "let a = true; let b = false; if (a) { if (b) { 1 } else { 2 } } else { 3 };",
    )
    .expect("compile should succeed");
    let ops = decode_instructions(&chunk)
        .iter()
        .map(|(_, op, _)| *op)
        .collect::<Vec<_>>();

    assert!(ops.iter().filter(|&&op| op == Opcode::JumpIfFalse).count() >= 2);
    assert!(ops.iter().filter(|&&op| op == Opcode::Jump).count() >= 2);
    assert_eq!(ops.last(), Some(&Opcode::ReturnValue));
}

#[test]
fn while_basic_compilation_shape() {
    let chunk = compile_input("while (true) { 1; }").expect("compile should succeed");
    let decoded = decode_instructions(&chunk);
    let ops = decoded.iter().map(|(_, op, _)| *op).collect::<Vec<_>>();

    assert!(ops.contains(&Opcode::JumpIfFalse));
    assert!(ops.contains(&Opcode::Jump));
    assert_eq!(ops.last(), Some(&Opcode::Return));

    let loop_back = decoded
        .iter()
        .filter(|(_, op, _)| *op == Opcode::Jump)
        .find(|(_, _, operands)| operands.first() == Some(&0))
        .is_some();
    assert!(loop_back);
}

#[test]
fn break_inside_loop_patches_to_loop_end() {
    let chunk = compile_input("while (true) { break; }").expect("compile should succeed");
    let decoded = decode_instructions(&chunk);

    let break_jump = decoded
        .iter()
        .find(|(offset, op, operands)| {
            *op == Opcode::Jump && operands.first().copied().unwrap_or_default() > *offset
        })
        .expect("expected break jump");
    let loop_end = decoded
        .iter()
        .find(|(_, op, _)| *op == Opcode::Return)
        .map(|(offset, _, _)| *offset)
        .expect("expected terminal Return");

    assert_eq!(break_jump.2[0], loop_end);
    assert!(!decoded.iter().any(|(_, op, _)| *op == Opcode::InvalidBreak));
}

#[test]
fn continue_inside_loop_jumps_to_loop_start() {
    let chunk = compile_input("while (true) { continue; }").expect("compile should succeed");
    let decoded = decode_instructions(&chunk);

    let continue_jump = decoded
        .iter()
        .find(|(_, op, operands)| *op == Opcode::Jump && operands.first() == Some(&0))
        .expect("expected continue jump to loop start");
    assert_eq!(continue_jump.2[0], 0);
    assert!(!decoded
        .iter()
        .any(|(_, op, _)| *op == Opcode::InvalidContinue));
}

#[test]
fn nested_loops_scope_break_continue_to_innermost() {
    let input = "let a = true; let b = true; while (a) { while (b) { break; } continue; }";
    let chunk = compile_input(input).expect("compile should succeed");
    let decoded = decode_instructions(&chunk);

    let loop_starts = decoded
        .iter()
        .filter(|(_, op, _)| *op == Opcode::GetGlobal)
        .map(|(offset, _, _)| *offset)
        .collect::<Vec<_>>();
    assert!(loop_starts.len() >= 2);

    let jumps = decoded
        .iter()
        .filter(|(_, op, _)| *op == Opcode::Jump)
        .map(|(offset, _, operands)| (*offset, operands[0]))
        .collect::<Vec<_>>();

    assert!(jumps.iter().any(|(_, target)| *target == loop_starts[0]));
    assert!(jumps.iter().any(|(_, target)| *target == loop_starts[1]));
}

#[test]
fn top_level_break_continue_emit_invalid_opcodes() {
    let chunk = compile_input("break;").expect("compile should succeed");
    let ops = decode_instructions(&chunk)
        .iter()
        .map(|(_, op, _)| *op)
        .collect::<Vec<_>>();
    assert!(ops.contains(&Opcode::InvalidBreak));
    assert_eq!(ops.last(), Some(&Opcode::Return));

    let chunk = compile_input("continue;").expect("compile should succeed");
    let ops = decode_instructions(&chunk)
        .iter()
        .map(|(_, op, _)| *op)
        .collect::<Vec<_>>();
    assert!(ops.contains(&Opcode::InvalidContinue));
    assert_eq!(ops.last(), Some(&Opcode::Return));
}

#[test]
fn position_metadata_for_new_control_flow_ops() {
    let input =
        "let a = true;\nlet b = true;\nif (a) { 1 };\nwhile (b) {\n  continue;\n}\nbreak;\n";
    let chunk = compile_input(input).expect("compile should succeed");
    let decoded = decode_instructions(&chunk);

    let mut saw_if_jump = false;
    let mut saw_if_null = false;
    let mut saw_while_jump = false;
    let mut saw_continue_jump = false;
    let mut saw_invalid_break = false;

    for (offset, op, _) in decoded {
        let pos = chunk
            .position_for_offset(offset)
            .expect("expected position metadata");
        if op == Opcode::JumpIfFalse && pos.line == 3 {
            saw_if_jump = true;
        }
        if op == Opcode::Null && pos.line == 3 {
            saw_if_null = true;
        }
        if op == Opcode::JumpIfFalse && pos.line == 4 {
            saw_while_jump = true;
        }
        if op == Opcode::Jump && pos.line == 5 {
            saw_continue_jump = true;
        }
        if op == Opcode::InvalidBreak && pos == Position::new(7, 1) {
            saw_invalid_break = true;
        }
    }

    assert!(saw_if_jump);
    assert!(saw_if_null);
    assert!(saw_while_jump);
    assert!(saw_continue_jump);
    assert!(saw_invalid_break);
}
