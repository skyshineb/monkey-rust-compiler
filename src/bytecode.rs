use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::object::ObjectRef;
use crate::position::Position;

pub type Instructions = Vec<u8>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opcode {
    Constant = 0,
    True = 1,
    False = 2,
    Null = 3,
    Pop = 4,
    Add = 5,
    Sub = 6,
    Mul = 7,
    Div = 8,
    Neg = 9,
    Bang = 10,
    Eq = 11,
    Ne = 12,
    Lt = 13,
    Gt = 14,
    Le = 15,
    Ge = 16,
    Jump = 17,
    JumpIfFalse = 18,
    GetGlobal = 19,
    SetGlobal = 20,
    GetLocal = 21,
    SetLocal = 22,
    GetBuiltin = 23,
    GetFree = 24,
    Closure = 25,
    CurrentClosure = 26,
    Call = 27,
    ReturnValue = 28,
    Return = 29,
    Array = 30,
    Hash = 31,
    Index = 32,
    InvalidBreak = 33,
    InvalidContinue = 34,
    Nop = 35,
}

const ALL_OPCODES: [Opcode; 36] = [
    Opcode::Constant,
    Opcode::True,
    Opcode::False,
    Opcode::Null,
    Opcode::Pop,
    Opcode::Add,
    Opcode::Sub,
    Opcode::Mul,
    Opcode::Div,
    Opcode::Neg,
    Opcode::Bang,
    Opcode::Eq,
    Opcode::Ne,
    Opcode::Lt,
    Opcode::Gt,
    Opcode::Le,
    Opcode::Ge,
    Opcode::Jump,
    Opcode::JumpIfFalse,
    Opcode::GetGlobal,
    Opcode::SetGlobal,
    Opcode::GetLocal,
    Opcode::SetLocal,
    Opcode::GetBuiltin,
    Opcode::GetFree,
    Opcode::Closure,
    Opcode::CurrentClosure,
    Opcode::Call,
    Opcode::ReturnValue,
    Opcode::Return,
    Opcode::Array,
    Opcode::Hash,
    Opcode::Index,
    Opcode::InvalidBreak,
    Opcode::InvalidContinue,
    Opcode::Nop,
];

impl Opcode {
    pub fn all() -> &'static [Opcode] {
        &ALL_OPCODES
    }

    pub fn to_byte(self) -> u8 {
        self as u8
    }

    pub fn from_byte(byte: u8) -> Option<Opcode> {
        match byte {
            0 => Some(Opcode::Constant),
            1 => Some(Opcode::True),
            2 => Some(Opcode::False),
            3 => Some(Opcode::Null),
            4 => Some(Opcode::Pop),
            5 => Some(Opcode::Add),
            6 => Some(Opcode::Sub),
            7 => Some(Opcode::Mul),
            8 => Some(Opcode::Div),
            9 => Some(Opcode::Neg),
            10 => Some(Opcode::Bang),
            11 => Some(Opcode::Eq),
            12 => Some(Opcode::Ne),
            13 => Some(Opcode::Lt),
            14 => Some(Opcode::Gt),
            15 => Some(Opcode::Le),
            16 => Some(Opcode::Ge),
            17 => Some(Opcode::Jump),
            18 => Some(Opcode::JumpIfFalse),
            19 => Some(Opcode::GetGlobal),
            20 => Some(Opcode::SetGlobal),
            21 => Some(Opcode::GetLocal),
            22 => Some(Opcode::SetLocal),
            23 => Some(Opcode::GetBuiltin),
            24 => Some(Opcode::GetFree),
            25 => Some(Opcode::Closure),
            26 => Some(Opcode::CurrentClosure),
            27 => Some(Opcode::Call),
            28 => Some(Opcode::ReturnValue),
            29 => Some(Opcode::Return),
            30 => Some(Opcode::Array),
            31 => Some(Opcode::Hash),
            32 => Some(Opcode::Index),
            33 => Some(Opcode::InvalidBreak),
            34 => Some(Opcode::InvalidContinue),
            35 => Some(Opcode::Nop),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    pub name: &'static str,
    pub operand_widths: &'static [usize],
}

const DEF_CONSTANT: Definition = Definition {
    name: "Constant",
    operand_widths: &[2],
};
const DEF_TRUE: Definition = Definition {
    name: "True",
    operand_widths: &[],
};
const DEF_FALSE: Definition = Definition {
    name: "False",
    operand_widths: &[],
};
const DEF_NULL: Definition = Definition {
    name: "Null",
    operand_widths: &[],
};
const DEF_POP: Definition = Definition {
    name: "Pop",
    operand_widths: &[],
};
const DEF_ADD: Definition = Definition {
    name: "Add",
    operand_widths: &[],
};
const DEF_SUB: Definition = Definition {
    name: "Sub",
    operand_widths: &[],
};
const DEF_MUL: Definition = Definition {
    name: "Mul",
    operand_widths: &[],
};
const DEF_DIV: Definition = Definition {
    name: "Div",
    operand_widths: &[],
};
const DEF_NEG: Definition = Definition {
    name: "Neg",
    operand_widths: &[],
};
const DEF_BANG: Definition = Definition {
    name: "Bang",
    operand_widths: &[],
};
const DEF_EQ: Definition = Definition {
    name: "Eq",
    operand_widths: &[],
};
const DEF_NE: Definition = Definition {
    name: "Ne",
    operand_widths: &[],
};
const DEF_LT: Definition = Definition {
    name: "Lt",
    operand_widths: &[],
};
const DEF_GT: Definition = Definition {
    name: "Gt",
    operand_widths: &[],
};
const DEF_LE: Definition = Definition {
    name: "Le",
    operand_widths: &[],
};
const DEF_GE: Definition = Definition {
    name: "Ge",
    operand_widths: &[],
};
const DEF_JUMP: Definition = Definition {
    name: "Jump",
    operand_widths: &[2],
};
const DEF_JUMP_IF_FALSE: Definition = Definition {
    name: "JumpIfFalse",
    operand_widths: &[2],
};
const DEF_GET_GLOBAL: Definition = Definition {
    name: "GetGlobal",
    operand_widths: &[2],
};
const DEF_SET_GLOBAL: Definition = Definition {
    name: "SetGlobal",
    operand_widths: &[2],
};
const DEF_GET_LOCAL: Definition = Definition {
    name: "GetLocal",
    operand_widths: &[1],
};
const DEF_SET_LOCAL: Definition = Definition {
    name: "SetLocal",
    operand_widths: &[1],
};
const DEF_GET_BUILTIN: Definition = Definition {
    name: "GetBuiltin",
    operand_widths: &[1],
};
const DEF_GET_FREE: Definition = Definition {
    name: "GetFree",
    operand_widths: &[1],
};
const DEF_CLOSURE: Definition = Definition {
    name: "Closure",
    operand_widths: &[2, 1],
};
const DEF_CURRENT_CLOSURE: Definition = Definition {
    name: "CurrentClosure",
    operand_widths: &[],
};
const DEF_CALL: Definition = Definition {
    name: "Call",
    operand_widths: &[1],
};
const DEF_RETURN_VALUE: Definition = Definition {
    name: "ReturnValue",
    operand_widths: &[],
};
const DEF_RETURN: Definition = Definition {
    name: "Return",
    operand_widths: &[],
};
const DEF_ARRAY: Definition = Definition {
    name: "Array",
    operand_widths: &[2],
};
const DEF_HASH: Definition = Definition {
    name: "Hash",
    operand_widths: &[2],
};
const DEF_INDEX: Definition = Definition {
    name: "Index",
    operand_widths: &[],
};
const DEF_INVALID_BREAK: Definition = Definition {
    name: "InvalidBreak",
    operand_widths: &[],
};
const DEF_INVALID_CONTINUE: Definition = Definition {
    name: "InvalidContinue",
    operand_widths: &[],
};
const DEF_NOP: Definition = Definition {
    name: "Nop",
    operand_widths: &[],
};

pub fn lookup_definition(op: Opcode) -> &'static Definition {
    match op {
        Opcode::Constant => &DEF_CONSTANT,
        Opcode::True => &DEF_TRUE,
        Opcode::False => &DEF_FALSE,
        Opcode::Null => &DEF_NULL,
        Opcode::Pop => &DEF_POP,
        Opcode::Add => &DEF_ADD,
        Opcode::Sub => &DEF_SUB,
        Opcode::Mul => &DEF_MUL,
        Opcode::Div => &DEF_DIV,
        Opcode::Neg => &DEF_NEG,
        Opcode::Bang => &DEF_BANG,
        Opcode::Eq => &DEF_EQ,
        Opcode::Ne => &DEF_NE,
        Opcode::Lt => &DEF_LT,
        Opcode::Gt => &DEF_GT,
        Opcode::Le => &DEF_LE,
        Opcode::Ge => &DEF_GE,
        Opcode::Jump => &DEF_JUMP,
        Opcode::JumpIfFalse => &DEF_JUMP_IF_FALSE,
        Opcode::GetGlobal => &DEF_GET_GLOBAL,
        Opcode::SetGlobal => &DEF_SET_GLOBAL,
        Opcode::GetLocal => &DEF_GET_LOCAL,
        Opcode::SetLocal => &DEF_SET_LOCAL,
        Opcode::GetBuiltin => &DEF_GET_BUILTIN,
        Opcode::GetFree => &DEF_GET_FREE,
        Opcode::Closure => &DEF_CLOSURE,
        Opcode::CurrentClosure => &DEF_CURRENT_CLOSURE,
        Opcode::Call => &DEF_CALL,
        Opcode::ReturnValue => &DEF_RETURN_VALUE,
        Opcode::Return => &DEF_RETURN,
        Opcode::Array => &DEF_ARRAY,
        Opcode::Hash => &DEF_HASH,
        Opcode::Index => &DEF_INDEX,
        Opcode::InvalidBreak => &DEF_INVALID_BREAK,
        Opcode::InvalidContinue => &DEF_INVALID_CONTINUE,
        Opcode::Nop => &DEF_NOP,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BytecodeError {
    UnknownOpcodeByte(u8),
    WrongOperandCount {
        opcode: Opcode,
        expected: usize,
        got: usize,
    },
    OperandOutOfRange {
        opcode: Opcode,
        index: usize,
        width: usize,
        value: usize,
    },
    TruncatedInstruction {
        opcode: Opcode,
        needed: usize,
        available: usize,
    },
}

impl Display for BytecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            BytecodeError::UnknownOpcodeByte(byte) => write!(f, "unknown opcode byte {byte}"),
            BytecodeError::WrongOperandCount {
                opcode,
                expected,
                got,
            } => write!(
                f,
                "wrong operand count for {}: expected {}, got {}",
                lookup_definition(*opcode).name,
                expected,
                got
            ),
            BytecodeError::OperandOutOfRange {
                opcode,
                index,
                width,
                value,
            } => write!(
                f,
                "operand out of range for {} operand #{} (width {}): {}",
                lookup_definition(*opcode).name,
                index,
                width,
                value
            ),
            BytecodeError::TruncatedInstruction {
                opcode,
                needed,
                available,
            } => write!(
                f,
                "truncated instruction for {}: needed {}, available {}",
                lookup_definition(*opcode).name,
                needed,
                available
            ),
        }
    }
}

pub fn make(op: Opcode, operands: &[usize]) -> Result<Vec<u8>, BytecodeError> {
    let def = lookup_definition(op);
    if operands.len() != def.operand_widths.len() {
        return Err(BytecodeError::WrongOperandCount {
            opcode: op,
            expected: def.operand_widths.len(),
            got: operands.len(),
        });
    }

    let mut out = Vec::with_capacity(1 + def.operand_widths.iter().sum::<usize>());
    out.push(op.to_byte());

    for (idx, (&value, &width)) in operands.iter().zip(def.operand_widths.iter()).enumerate() {
        match width {
            1 => {
                if value > u8::MAX as usize {
                    return Err(BytecodeError::OperandOutOfRange {
                        opcode: op,
                        index: idx,
                        width,
                        value,
                    });
                }
                out.push(value as u8);
            }
            2 => {
                if value > u16::MAX as usize {
                    return Err(BytecodeError::OperandOutOfRange {
                        opcode: op,
                        index: idx,
                        width,
                        value,
                    });
                }
                let bytes = (value as u16).to_be_bytes();
                out.extend_from_slice(&bytes);
            }
            _ => {
                return Err(BytecodeError::OperandOutOfRange {
                    opcode: op,
                    index: idx,
                    width,
                    value,
                });
            }
        }
    }

    Ok(out)
}

pub fn read_operands(def: &Definition, bytes: &[u8]) -> Result<(Vec<usize>, usize), BytecodeError> {
    let mut operands = Vec::with_capacity(def.operand_widths.len());
    let mut offset = 0;

    for &width in def.operand_widths {
        if offset + width > bytes.len() {
            return Err(BytecodeError::TruncatedInstruction {
                opcode: Opcode::Nop,
                needed: offset + width,
                available: bytes.len(),
            });
        }

        let value = match width {
            1 => bytes[offset] as usize,
            2 => u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize,
            _ => {
                return Err(BytecodeError::TruncatedInstruction {
                    opcode: Opcode::Nop,
                    needed: offset + width,
                    available: bytes.len(),
                });
            }
        };
        operands.push(value);
        offset += width;
    }

    Ok((operands, offset))
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub instructions: Instructions,
    pub constants: Vec<ObjectRef>,
    pub positions: Vec<(usize, Position)>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty() -> Self {
        Self::new()
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
        let offset = self.instructions.len();
        self.instructions.extend_from_slice(bytes);
        offset
    }

    pub fn add_constant(&mut self, obj: ObjectRef) -> usize {
        let idx = self.constants.len();
        self.constants.push(obj);
        idx
    }

    pub fn record_pos(&mut self, offset: usize, pos: Position) {
        self.positions.push((offset, pos));
        self.positions.sort_by_key(|(off, _)| *off);
    }

    pub fn position_for_offset(&self, offset: usize) -> Option<Position> {
        self.positions
            .iter()
            .take_while(|(off, _)| *off <= offset)
            .last()
            .map(|(_, pos)| *pos)
    }

    pub fn disassemble(&self) -> String {
        // TODO(step-10): compiler will emit chunk instructions and position metadata.
        // TODO(step-17): VM will consume offsets for runtime error source mapping.
        let mut lines = Vec::new();
        let mut offset = 0;

        while offset < self.instructions.len() {
            let byte = self.instructions[offset];
            let Some(op) = Opcode::from_byte(byte) else {
                lines.push(format!("{:04} <unknown opcode {}>", offset, byte));
                break;
            };

            let def = lookup_definition(op);
            let operands_start = offset + 1;
            let operand_bytes = &self.instructions[operands_start..];
            let decoded = read_operands(def, operand_bytes);

            match decoded {
                Ok((operands, consumed)) => {
                    let operands_rendered = if operands.is_empty() {
                        String::new()
                    } else {
                        format!(
                            " {}",
                            operands
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(" ")
                        )
                    };
                    let pos_suffix = self
                        .position_for_offset(offset)
                        .map(|p| format!(" @{}", p))
                        .unwrap_or_default();
                    lines.push(format!(
                        "{:04} {}{}{}",
                        offset, def.name, operands_rendered, pos_suffix
                    ));
                    offset += 1 + consumed;
                }
                Err(_) => {
                    lines.push(format!("{:04} {} <truncated>", offset, def.name));
                    break;
                }
            }
        }

        lines.join("\n")
    }
}

pub type Bytecode = Chunk;
