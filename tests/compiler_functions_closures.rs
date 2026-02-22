use std::rc::Rc;

use monkey_rust_compiler::ast::Program;
use monkey_rust_compiler::bytecode::{lookup_definition, read_operands, Chunk, Opcode};
use monkey_rust_compiler::compiler::{CompileError, Compiler};
use monkey_rust_compiler::lexer::Lexer;
use monkey_rust_compiler::object::{CompiledFunctionObject, Object};
use monkey_rust_compiler::parser::Parser;

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

#[test]
fn function_literal_compiles_to_closure_constant() {
    let chunk = compile_input("fn() { 1; };").expect("compile should succeed");
    let root = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();

    assert_eq!(root[0].0, Opcode::Closure);
    assert_eq!(root[0].1[1], 0);
    assert_eq!(root.last().map(|x| x.0), Some(Opcode::ReturnValue));

    let fn_const_idx = root[0].1[0];
    let function = as_compiled_function(&chunk.constants[fn_const_idx]);
    let body = decode_instructions(&function.instructions)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();
    assert_eq!(body, vec![Opcode::Constant, Opcode::ReturnValue]);
    assert_eq!(function.num_params, 0);
    assert_eq!(function.num_locals, 0);
}

#[test]
fn explicit_return_in_function_preserved() {
    let chunk = compile_input("fn() { return 1; };").expect("compile should succeed");
    let root = decode_chunk(&chunk);
    let fn_const_idx = root[0].2[0];
    let function = as_compiled_function(&chunk.constants[fn_const_idx]);
    let body = decode_instructions(&function.instructions)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();

    assert_eq!(body, vec![Opcode::Constant, Opcode::ReturnValue]);
}

#[test]
fn function_without_expression_emits_return() {
    let chunk = compile_input("fn() { let a = 1; };").expect("compile should succeed");
    let root = decode_chunk(&chunk);
    let fn_const_idx = root[0].2[0];
    let function = as_compiled_function(&chunk.constants[fn_const_idx]);
    let body = decode_instructions(&function.instructions)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();

    assert_eq!(body.last(), Some(&Opcode::Return));
}

#[test]
fn parameter_and_local_slot_usage() {
    let chunk = compile_input("fn(a) { a; };").expect("compile should succeed");
    let root = decode_chunk(&chunk);
    let function = as_compiled_function(&chunk.constants[root[0].2[0]]);
    let body = decode_instructions(&function.instructions)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert_eq!(
        body,
        vec![(Opcode::GetLocal, vec![0]), (Opcode::ReturnValue, vec![])]
    );

    let chunk = compile_input("fn(a, b) { a + b; };").expect("compile should succeed");
    let root = decode_chunk(&chunk);
    let function = as_compiled_function(&chunk.constants[root[0].2[0]]);
    let body = decode_instructions(&function.instructions)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert_eq!(
        body,
        vec![
            (Opcode::GetLocal, vec![0]),
            (Opcode::GetLocal, vec![1]),
            (Opcode::Add, vec![]),
            (Opcode::ReturnValue, vec![]),
        ]
    );

    let chunk = compile_input("fn(a) { let b = 1; a + b; };").expect("compile should succeed");
    let root = decode_chunk(&chunk);
    let function = as_compiled_function(&chunk.constants[root[0].2[0]]);
    let body = decode_instructions(&function.instructions)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert!(body.contains(&(Opcode::SetLocal, vec![1])));
    assert!(body.contains(&(Opcode::GetLocal, vec![1])));
}

#[test]
fn call_expression_compilation_order_and_argc() {
    let chunk =
        compile_input("let add = fn(a, b) { a + b; }; add(1, 2);").expect("compile should succeed");
    let ops = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();

    let call_idx = ops
        .iter()
        .position(|(op, _)| *op == Opcode::Call)
        .expect("expected call opcode");
    assert_eq!(ops[call_idx], (Opcode::Call, vec![2]));
    assert_eq!(ops[call_idx - 3].0, Opcode::GetGlobal);
    assert_eq!(ops[call_idx - 2].0, Opcode::Constant);
    assert_eq!(ops[call_idx - 1].0, Opcode::Constant);

    let chunk = compile_input("fn(x) { x }(5);").expect("compile should succeed");
    let ops = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert!(ops.iter().any(|(op, _)| *op == Opcode::Closure));
    assert!(ops
        .iter()
        .any(|(op, args)| *op == Opcode::Call && args == &vec![1]));
}

#[test]
fn builtin_call_inside_compiler_pipeline() {
    let chunk = compile_input("len(\"abc\");").expect("compile should succeed");
    let ops = decode_chunk(&chunk)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();

    assert!(ops.contains(&(Opcode::GetBuiltin, vec![0])));
    assert!(ops
        .iter()
        .any(|(op, args)| *op == Opcode::Call && args == &vec![1]));
}

#[test]
fn closure_free_variable_capture() {
    let chunk = compile_input("fn(a) { fn(b) { a + b } };").expect("compile should succeed");

    let compiled_functions = chunk
        .constants
        .iter()
        .filter_map(|obj| match obj.as_ref() {
            Object::CompiledFunction(f) => Some(Rc::clone(f)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(compiled_functions.len() >= 2);

    let inner = compiled_functions
        .iter()
        .find(|f| f.num_params == 1 && f.num_locals == 1)
        .expect("expected inner function");
    let inner_ops = decode_instructions(&inner.instructions)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert!(inner_ops.contains(&(Opcode::GetFree, vec![0])));
    assert!(inner_ops.contains(&(Opcode::GetLocal, vec![0])));

    let outer = compiled_functions
        .iter()
        .find(|f| {
            f.num_params == 1
                && f.num_locals >= 1
                && f.instructions.len() > inner.instructions.len()
        })
        .expect("expected outer function");
    let outer_ops = decode_instructions(&outer.instructions)
        .into_iter()
        .map(|(_, op, operands)| (op, operands))
        .collect::<Vec<_>>();
    assert!(outer_ops.contains(&(Opcode::GetLocal, vec![0])));
    assert!(outer_ops
        .iter()
        .any(|(op, ops)| *op == Opcode::Closure && ops[1] == 1));
}

#[test]
fn nested_closure_chain_capture() {
    let input = "fn(a) { fn(b) { fn(c) { a + b + c } } };";
    let chunk = compile_input(input).expect("compile should succeed");

    let closures = decode_chunk(&chunk)
        .into_iter()
        .filter(|(_, op, _)| *op == Opcode::Closure)
        .collect::<Vec<_>>();
    assert!(!closures.is_empty());

    let compiled_functions = chunk
        .constants
        .iter()
        .filter_map(|obj| match obj.as_ref() {
            Object::CompiledFunction(f) => Some(Rc::clone(f)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(compiled_functions.len() >= 3);

    let deepest = compiled_functions
        .iter()
        .find(|f| f.num_params == 1 && f.instructions.len() >= 6)
        .expect("expected deepest function");
    let ops = decode_instructions(&deepest.instructions)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();
    assert!(ops.contains(&Opcode::GetFree));
    assert!(ops.contains(&Opcode::GetLocal));
}

#[test]
fn recursive_function_self_reference_via_current_closure() {
    let input = "let fact = fn(n) { if (n == 0) { 1 } else { fact(n - 1) } };";
    let chunk = compile_input(input).expect("compile should succeed");

    let function = chunk
        .constants
        .iter()
        .find_map(|obj| match obj.as_ref() {
            Object::CompiledFunction(f) => Some(Rc::clone(f)),
            _ => None,
        })
        .expect("expected compiled function");
    let ops = decode_instructions(&function.instructions)
        .into_iter()
        .map(|(_, op, _)| op)
        .collect::<Vec<_>>();

    assert!(ops.contains(&Opcode::CurrentClosure));
    assert!(ops.contains(&Opcode::Call));
}

#[test]
fn position_metadata_for_closure_and_call() {
    let input = "let add = fn(a, b) {\n  a + b\n};\nadd(1, 2);\n";
    let chunk = compile_input(input).expect("compile should succeed");
    let decoded = decode_chunk(&chunk);

    let closure_offset = decoded
        .iter()
        .find(|(_, op, _)| *op == Opcode::Closure)
        .map(|(offset, _, _)| *offset)
        .expect("expected closure instruction");
    let call_offset = decoded
        .iter()
        .find(|(_, op, _)| *op == Opcode::Call)
        .map(|(offset, _, _)| *offset)
        .expect("expected call instruction");

    assert!(chunk.position_for_offset(closure_offset).is_some());
    assert!(chunk.position_for_offset(call_offset).is_some());

    let fn_const_idx = decoded
        .iter()
        .find(|(_, op, _)| *op == Opcode::Closure)
        .map(|(_, _, operands)| operands[0])
        .expect("expected closure operand");
    let function = as_compiled_function(&chunk.constants[fn_const_idx]);
    assert!(!function.positions.is_empty());
}

#[test]
fn collections_index_still_unsupported() {
    let cases = [
        ("[1,2]", "unsupported expression in step 14: ArrayLiteral"),
        (
            "{\"a\":1}",
            "unsupported expression in step 14: HashLiteral",
        ),
        ("arr[0]", "unsupported expression in step 14: Index"),
    ];

    for (input, expected) in cases {
        let err = compile_input(input).expect_err("expected compile error");
        assert_eq!(err.message, expected, "input={input}");
        assert!(err.pos.is_some(), "input={input}");
    }
}
