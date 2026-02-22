use monkey_rust_compiler::bytecode::{
    lookup_definition, make, read_operands, Bytecode, BytecodeError, Opcode,
};
use monkey_rust_compiler::object::Object;
use monkey_rust_compiler::position::Position;

#[test]
fn opcode_roundtrip_and_unknown_byte_behavior() {
    for &op in Opcode::all() {
        assert_eq!(Opcode::from_byte(op.to_byte()), Some(op));
    }

    assert_eq!(Opcode::from_byte(255), None);
}

#[test]
fn opcode_definition_metadata_is_stable() {
    let constant = lookup_definition(Opcode::Constant);
    assert_eq!(constant.name, "Constant");
    assert_eq!(constant.operand_widths, &[2]);

    let jump = lookup_definition(Opcode::Jump);
    assert_eq!(jump.name, "Jump");
    assert_eq!(jump.operand_widths, &[2]);

    let jump_if_false = lookup_definition(Opcode::JumpIfFalse);
    assert_eq!(jump_if_false.name, "JumpIfFalse");
    assert_eq!(jump_if_false.operand_widths, &[2]);

    let call = lookup_definition(Opcode::Call);
    assert_eq!(call.name, "Call");
    assert_eq!(call.operand_widths, &[1]);

    let closure = lookup_definition(Opcode::Closure);
    assert_eq!(closure.name, "Closure");
    assert_eq!(closure.operand_widths, &[2, 1]);

    let current_closure = lookup_definition(Opcode::CurrentClosure);
    assert_eq!(current_closure.name, "CurrentClosure");
    assert_eq!(current_closure.operand_widths, &[]);

    let add = lookup_definition(Opcode::Add);
    assert_eq!(add.name, "Add");
    assert_eq!(add.operand_widths, &[]);

    let ret = lookup_definition(Opcode::Return);
    assert_eq!(ret.name, "Return");
    assert_eq!(ret.operand_widths, &[]);
}

#[test]
fn make_encodes_instructions_deterministically() {
    let add = make(Opcode::Add, &[]).expect("encode add");
    assert_eq!(add, vec![Opcode::Add.to_byte()]);

    let call = make(Opcode::Call, &[3]).expect("encode call");
    assert_eq!(call, vec![Opcode::Call.to_byte(), 3]);

    let constant = make(Opcode::Constant, &[655]).expect("encode constant");
    assert_eq!(constant, vec![Opcode::Constant.to_byte(), 0x02, 0x8F]);

    let closure = make(Opcode::Closure, &[10, 2]).expect("encode closure");
    assert_eq!(closure, vec![Opcode::Closure.to_byte(), 0x00, 0x0A, 0x02]);
}

#[test]
fn read_operands_decodes_values_and_consumed_length() {
    let def_constant = lookup_definition(Opcode::Constant);
    let (operands, consumed) = read_operands(def_constant, &[0x02, 0x8F]).expect("decode constant");
    assert_eq!(operands, vec![655]);
    assert_eq!(consumed, 2);

    let def_call = lookup_definition(Opcode::Call);
    let (operands, consumed) = read_operands(def_call, &[3]).expect("decode call");
    assert_eq!(operands, vec![3]);
    assert_eq!(consumed, 1);

    let def_closure = lookup_definition(Opcode::Closure);
    let (operands, consumed) =
        read_operands(def_closure, &[0x00, 0x0A, 0x02]).expect("decode closure");
    assert_eq!(operands, vec![10, 2]);
    assert_eq!(consumed, 3);
}

#[test]
fn encoding_and_decoding_errors_are_deterministic() {
    let err = make(Opcode::Call, &[]).expect_err("should error on operand count");
    assert_eq!(
        err,
        BytecodeError::WrongOperandCount {
            opcode: Opcode::Call,
            expected: 1,
            got: 0,
        }
    );

    let err = make(Opcode::Call, &[300]).expect_err("should error on u8 range");
    assert_eq!(
        err,
        BytecodeError::OperandOutOfRange {
            opcode: Opcode::Call,
            index: 0,
            width: 1,
            value: 300,
        }
    );

    let err = read_operands(lookup_definition(Opcode::Constant), &[0x01])
        .expect_err("should error on truncated bytes");
    assert_eq!(
        err,
        BytecodeError::TruncatedInstruction {
            opcode: Opcode::Nop,
            needed: 2,
            available: 1,
        }
    );

    assert_eq!(Opcode::from_byte(254), None);
}

#[test]
fn chunk_constant_pool_preserves_indices_and_order() {
    let mut chunk = Bytecode::new();

    let i0 = chunk.add_constant(Object::Integer(1).rc());
    let i1 = chunk.add_constant(Object::String("hello".to_string()).rc());
    let i2 = chunk.add_constant(Object::Boolean(true).rc());

    assert_eq!(i0, 0);
    assert_eq!(i1, 1);
    assert_eq!(i2, 2);

    assert_eq!(chunk.constants.len(), 3);
    assert_eq!(*chunk.constants[0], Object::Integer(1));
    assert_eq!(*chunk.constants[1], Object::String("hello".to_string()));
    assert_eq!(*chunk.constants[2], Object::Boolean(true));
}

#[test]
fn chunk_position_mapping_uses_nearest_lower_offset() {
    let mut chunk = Bytecode::new();
    chunk.record_pos(0, Position::new(1, 1));
    chunk.record_pos(3, Position::new(2, 5));
    chunk.record_pos(8, Position::new(4, 2));

    assert_eq!(chunk.position_for_offset(0), Some(Position::new(1, 1)));
    assert_eq!(chunk.position_for_offset(3), Some(Position::new(2, 5)));
    assert_eq!(chunk.position_for_offset(5), Some(Position::new(2, 5)));
    assert_eq!(chunk.position_for_offset(8), Some(Position::new(4, 2)));
    assert_eq!(chunk.position_for_offset(100), Some(Position::new(4, 2)));

    let mut empty = Bytecode::new();
    empty.record_pos(4, Position::new(9, 9));
    assert_eq!(empty.position_for_offset(2), None);
}

#[test]
fn disassembler_output_is_deterministic() {
    let mut chunk = Bytecode::new();

    let off0 = chunk.push_bytes(&make(Opcode::Constant, &[1]).expect("constant"));
    chunk.record_pos(off0, Position::new(1, 1));

    let off1 = chunk.push_bytes(&make(Opcode::Add, &[]).expect("add"));
    chunk.record_pos(off1, Position::new(1, 5));

    let off2 = chunk.push_bytes(&make(Opcode::JumpIfFalse, &[12]).expect("jump-if-false"));
    chunk.record_pos(off2, Position::new(2, 3));

    let expected = "0000 Constant 1 @1:1\n0003 Add @1:5\n0004 JumpIfFalse 12 @2:3";
    assert_eq!(chunk.disassemble(), expected);
}
