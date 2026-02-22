use std::rc::Rc;

use monkey_rust_compiler::ast::Program;
use monkey_rust_compiler::bytecode::{lookup_definition, read_operands, Chunk, Opcode};
use monkey_rust_compiler::compiler::{CompileError, Compiler};
use monkey_rust_compiler::lexer::Lexer;
use monkey_rust_compiler::object::{CompiledFunctionObject, Object};
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

fn decode_instructions(bytes: &[u8]) -> Vec<(usize, Opcode, Vec<usize>)> {
    let mut out = Vec::new();
    let mut offset = 0;

    while offset < bytes.len() {
        let op = Opcode::from_byte(bytes[offset])
            .unwrap_or_else(|| panic!("unknown opcode at offset {offset}"));
        let def = lookup_definition(op);
        let (operands, consumed) = read_operands(def, &bytes[offset + 1..])
            .unwrap_or_else(|err| panic!("failed decoding operands at {offset}: {err}"));
        out.push((offset, op, operands));
        offset += 1 + consumed;
    }

    out
}

fn decode_chunk(chunk: &Chunk) -> Vec<(usize, Opcode, Vec<usize>)> {
    decode_instructions(&chunk.instructions)
}

fn as_compiled_function(obj: &Rc<Object>) -> Rc<CompiledFunctionObject> {
    match obj.as_ref() {
        Object::CompiledFunction(f) => Rc::clone(f),
        other => panic!("expected compiled function constant, got {other:?}"),
    }
}

fn chunk_constants(chunk: &Chunk) -> Vec<Object> {
    chunk.constants.iter().map(|c| c.as_ref().clone()).collect()
}

fn function_pos_for_offset(function: &CompiledFunctionObject, offset: usize) -> Option<Position> {
    function
        .positions
        .iter()
        .filter(|(pos_offset, _)| *pos_offset <= offset)
        .max_by_key(|(pos_offset, _)| *pos_offset)
        .map(|(_, pos)| *pos)
}

#[test]
fn array_literal_compilation_basic() {
    let chunk = compile_input("[];").expect("compile should succeed");
    let decoded = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert_eq!(
        decoded,
        vec![(Opcode::Array, vec![0]), (Opcode::ReturnValue, vec![])]
    );
    assert!(chunk.constants.is_empty());

    let chunk = compile_input("[1, 2, 3];").expect("compile should succeed");
    let decoded = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert_eq!(
        decoded,
        vec![
            (Opcode::Constant, vec![0]),
            (Opcode::Constant, vec![1]),
            (Opcode::Constant, vec![2]),
            (Opcode::Array, vec![3]),
            (Opcode::ReturnValue, vec![]),
        ]
    );
    assert_eq!(
        chunk_constants(&chunk),
        vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)]
    );

    let chunk = compile_input("[1 + 2, 3 * 4, 5];").expect("compile should succeed");
    let ops = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();
    assert_eq!(
        ops,
        vec![
            Opcode::Constant,
            Opcode::Constant,
            Opcode::Add,
            Opcode::Constant,
            Opcode::Constant,
            Opcode::Mul,
            Opcode::Constant,
            Opcode::Array,
            Opcode::ReturnValue,
        ]
    );
}

#[test]
fn hash_literal_compilation_basic() {
    let chunk = compile_input("{};").expect("compile should succeed");
    let decoded = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert_eq!(
        decoded,
        vec![(Opcode::Hash, vec![0]), (Opcode::ReturnValue, vec![])]
    );

    let chunk = compile_input("{\"one\": 1, \"two\": 2};").expect("compile should succeed");
    let decoded = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert_eq!(
        decoded,
        vec![
            (Opcode::Constant, vec![0]),
            (Opcode::Constant, vec![1]),
            (Opcode::Constant, vec![2]),
            (Opcode::Constant, vec![3]),
            (Opcode::Hash, vec![2]),
            (Opcode::ReturnValue, vec![]),
        ]
    );
    assert_eq!(
        chunk_constants(&chunk),
        vec![
            Object::String("one".to_string()),
            Object::Integer(1),
            Object::String("two".to_string()),
            Object::Integer(2),
        ]
    );

    let chunk = compile_input("{1 + 1: 2 * 2, \"x\": 3};").expect("compile should succeed");
    let ops = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();
    assert_eq!(
        ops,
        vec![
            Opcode::Constant,
            Opcode::Constant,
            Opcode::Add,
            Opcode::Constant,
            Opcode::Constant,
            Opcode::Mul,
            Opcode::Constant,
            Opcode::Constant,
            Opcode::Hash,
            Opcode::ReturnValue,
        ]
    );
}

#[test]
fn hash_source_order_preserved() {
    let chunk = compile_input("{\"b\": 2, \"a\": 1, \"c\": 3};").expect("compile should succeed");
    assert_eq!(
        chunk_constants(&chunk),
        vec![
            Object::String("b".to_string()),
            Object::Integer(2),
            Object::String("a".to_string()),
            Object::Integer(1),
            Object::String("c".to_string()),
            Object::Integer(3),
        ]
    );

    let decoded = decode_chunk(&chunk);
    let hash = decoded
        .iter()
        .find(|(_, op, _)| *op == Opcode::Hash)
        .expect("expected hash opcode");
    assert_eq!(hash.2, vec![3]);
}

#[test]
fn index_expression_compilation() {
    let chunk = compile_input("[1, 2, 3][1];").expect("compile should succeed");
    let ops = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();
    assert_eq!(
        ops,
        vec![
            Opcode::Constant,
            Opcode::Constant,
            Opcode::Constant,
            Opcode::Array,
            Opcode::Constant,
            Opcode::Index,
            Opcode::ReturnValue,
        ]
    );

    let chunk = compile_input("{\"a\": 1}[\"a\"];").expect("compile should succeed");
    let ops = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();
    assert!(ops.contains(&Opcode::Hash));
    assert!(ops.contains(&Opcode::Index));

    let chunk = compile_input("let arr = [1,2,3]; arr[1 + 1];").expect("compile should succeed");
    let ops = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();
    assert!(ops.contains(&Opcode::GetGlobal));
    assert!(ops.contains(&Opcode::Add));
    assert!(ops.contains(&Opcode::Index));
    assert_eq!(ops.last(), Some(&Opcode::ReturnValue));
}

#[test]
fn nested_collection_and_index_composition() {
    for input in [
        "[[1, 2], [3, 4]][0][1];",
        "{\"x\": [1, 2, 3]}[\"x\"][2];",
        "[fn(x) { x }(1), 2][0];",
    ] {
        let chunk = compile_input(input).expect("compile should succeed");
        let ops = decode_chunk(&chunk)
            .into_iter()
            .map(|(_, op, _)| op)
            .collect::<Vec<_>>();
        assert!(ops.contains(&Opcode::Array), "input={input}");
        assert!(ops.contains(&Opcode::Index), "input={input}");
        assert_eq!(ops.last(), Some(&Opcode::ReturnValue), "input={input}");
    }
}

#[test]
fn collections_inside_function_scope() {
    let input = "let make = fn(a) { let h = {\"x\": a}; [h[\"x\"], a]; }; make(5);";
    let chunk = compile_input(input).expect("compile should succeed");

    let root_ops = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();
    assert!(root_ops.contains(&Opcode::Closure));
    assert!(root_ops.contains(&Opcode::SetGlobal));
    assert!(root_ops.contains(&Opcode::GetGlobal));
    assert!(root_ops.contains(&Opcode::Call));

    let function = chunk
        .constants
        .iter()
        .find_map(|obj| match obj.as_ref() {
            Object::CompiledFunction(_) => Some(as_compiled_function(obj)),
            _ => None,
        })
        .expect("expected compiled function constant");
    let fn_ops = decode_instructions(&function.instructions)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();
    assert!(fn_ops.contains(&Opcode::Hash));
    assert!(fn_ops.contains(&Opcode::Index));
    assert!(fn_ops.contains(&Opcode::Array));
}

#[test]
fn runtime_validation_deferred_compiles_successfully() {
    for input in ["{[]: 1};", "1[0];", "{\"a\": 1}[[]];"] {
        compile_input(input).expect("compiler should defer runtime validation");
    }
}

#[test]
fn position_metadata_for_array_hash_index() {
    let chunk = compile_input("let a = [1, 2];\nlet h = {\"x\": a[0]};\nh[\"x\"];\n")
        .expect("compile should succeed");
    let decoded = decode_chunk(&chunk);

    let array_offsets = decoded
        .iter()
        .filter(|(_, op, _)| *op == Opcode::Array)
        .map(|(offset, _, _)| *offset)
        .collect::<Vec<_>>();
    let hash_offset = decoded
        .iter()
        .find(|(_, op, _)| *op == Opcode::Hash)
        .map(|(offset, _, _)| *offset)
        .expect("expected hash opcode");
    let index_offsets = decoded
        .iter()
        .filter(|(_, op, _)| *op == Opcode::Index)
        .map(|(offset, _, _)| *offset)
        .collect::<Vec<_>>();

    assert_eq!(
        chunk.position_for_offset(array_offsets[0]),
        Some(Position::new(1, 9))
    );
    assert_eq!(array_offsets.len(), 1);
    assert_eq!(
        chunk.position_for_offset(hash_offset),
        Some(Position::new(2, 9))
    );
    assert_eq!(
        chunk.position_for_offset(index_offsets[0]),
        Some(Position::new(2, 16))
    );
    assert_eq!(
        chunk.position_for_offset(index_offsets[1]),
        Some(Position::new(3, 2))
    );
}

#[test]
fn function_collection_positions_are_preserved() {
    let chunk = compile_input("let make = fn(a) { [a, {\"x\": a}[\"x\"]]; };")
        .expect("compile should succeed");
    let function = chunk
        .constants
        .iter()
        .find_map(|obj| match obj.as_ref() {
            Object::CompiledFunction(_) => Some(as_compiled_function(obj)),
            _ => None,
        })
        .expect("expected compiled function constant");
    let decoded = decode_instructions(&function.instructions);

    for (offset, op, _) in decoded {
        if matches!(op, Opcode::Array | Opcode::Hash | Opcode::Index) {
            assert!(
                function_pos_for_offset(&function, offset).is_some(),
                "expected function position for opcode {op:?} at offset {offset}"
            );
        }
    }
}
