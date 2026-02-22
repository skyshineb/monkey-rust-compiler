use std::rc::Rc;

use crate::builtins::{builtin_name_at, execute_builtin};
use crate::bytecode::{lookup_definition, Chunk, Opcode};
use crate::object::{ClosureObject, CompiledFunctionObject, Object, ObjectRef};
use crate::position::Position;
use crate::runtime_error::{RuntimeError, RuntimeErrorType, StackFrameInfo};

#[derive(Debug, Clone)]
struct Frame {
    closure: Rc<ClosureObject>,
    ip: usize,
    base_pointer: usize,
    call_site_pos: Position,
    arg_count: usize,
}

impl Frame {
    fn new(
        closure: Rc<ClosureObject>,
        base_pointer: usize,
        call_site_pos: Position,
        arg_count: usize,
    ) -> Self {
        Self {
            closure,
            ip: 0,
            base_pointer,
            call_site_pos,
            arg_count,
        }
    }
}

/// Stack-based VM for executing compiled Monkey bytecode.
#[derive(Debug, Clone)]
pub struct Vm {
    chunk: Chunk,
    stack: Vec<ObjectRef>,
    globals: Vec<ObjectRef>,
    frames: Vec<Frame>,
    last_popped: Option<ObjectRef>,
    output: Vec<String>,
}

impl Vm {
    pub fn new(chunk: Chunk) -> Self {
        let main_function = Rc::new(CompiledFunctionObject {
            name: Some("<repl>".to_string()),
            num_params: 0,
            num_locals: 0,
            instructions: chunk.instructions.clone(),
            positions: chunk.positions.clone(),
        });
        let main_closure = Rc::new(ClosureObject {
            function: main_function,
            free: Vec::new(),
        });

        Self {
            chunk,
            stack: Vec::new(),
            globals: Vec::new(),
            frames: vec![Frame::new(main_closure, 0, Position::default(), 0)],
            last_popped: None,
            output: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Result<ObjectRef, RuntimeError> {
        while !self.frames.is_empty() {
            let (ip, instr_len) = {
                let frame = self.current_frame().ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorType::UnsupportedOperation,
                        "frame stack underflow",
                        Position::default(),
                    )
                })?;
                (frame.ip, frame.closure.function.instructions.len())
            };

            if ip >= instr_len {
                if self.frames.len() == 1 {
                    return Ok(Object::Null.rc());
                }
                return Err(self.runtime_error(
                    ip,
                    RuntimeErrorType::UnsupportedOperation,
                    "reached end of function without return",
                ));
            }

            let opcode_byte = self.current_instructions()[ip];
            let Some(opcode) = Opcode::from_byte(opcode_byte) else {
                return Err(self.runtime_error(
                    ip,
                    RuntimeErrorType::UnsupportedOperation,
                    format!("unknown opcode byte: {opcode_byte}"),
                ));
            };

            match opcode {
                Opcode::Constant => {
                    let idx = self.read_u16_operand(ip)?;
                    let Some(constant) = self.chunk.constants.get(idx).cloned() else {
                        return Err(self.runtime_error(
                            ip,
                            RuntimeErrorType::UnsupportedOperation,
                            format!("constant index out of bounds: {idx}"),
                        ));
                    };
                    self.push(constant, ip)?;
                    self.advance_ip(3)?;
                }
                Opcode::True => {
                    self.push(Object::Boolean(true).rc(), ip)?;
                    self.advance_ip(1)?;
                }
                Opcode::False => {
                    self.push(Object::Boolean(false).rc(), ip)?;
                    self.advance_ip(1)?;
                }
                Opcode::Null => {
                    self.push(Object::Null.rc(), ip)?;
                    self.advance_ip(1)?;
                }
                Opcode::Pop => {
                    self.pop(ip)?;
                    self.advance_ip(1)?;
                }
                Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div => {
                    self.exec_binary_arithmetic(opcode, ip)?;
                    self.advance_ip(1)?;
                }
                Opcode::Neg => {
                    let operand = self.pop(ip)?;
                    let result = match operand.as_ref() {
                        Object::Integer(v) => Object::Integer(-v).rc(),
                        Object::Null => Object::Null.rc(),
                        other => {
                            return Err(self.runtime_error(
                                ip,
                                RuntimeErrorType::TypeMismatch,
                                format!("unsupported operand type for -: {}", other.type_name()),
                            ));
                        }
                    };
                    self.push(result, ip)?;
                    self.advance_ip(1)?;
                }
                Opcode::Bang => {
                    let operand = self.pop(ip)?;
                    self.push(Object::Boolean(!operand.as_ref().is_truthy()).rc(), ip)?;
                    self.advance_ip(1)?;
                }
                Opcode::Eq | Opcode::Ne | Opcode::Lt | Opcode::Gt | Opcode::Le | Opcode::Ge => {
                    self.exec_comparison(opcode, ip)?;
                    self.advance_ip(1)?;
                }
                Opcode::Jump => {
                    let target = self.read_u16_operand(ip)?;
                    self.ensure_jump_target(ip, target)?;
                    self.set_ip(target)?;
                }
                Opcode::JumpIfFalse => {
                    let target = self.read_u16_operand(ip)?;
                    self.ensure_jump_target(ip, target)?;
                    let condition = self.peek(ip)?;
                    if !condition.as_ref().is_truthy() {
                        self.set_ip(target)?;
                    } else {
                        self.advance_ip(3)?;
                    }
                }
                Opcode::SetGlobal => {
                    let idx = self.read_u16_operand(ip)?;
                    let value = self.pop(ip)?;
                    while self.globals.len() <= idx {
                        self.globals.push(Object::Null.rc());
                    }
                    self.globals[idx] = value;
                    self.advance_ip(3)?;
                }
                Opcode::GetGlobal => {
                    let idx = self.read_u16_operand(ip)?;
                    let Some(value) = self.globals.get(idx).cloned() else {
                        return Err(self.runtime_error(
                            ip,
                            RuntimeErrorType::UnknownIdentifier,
                            format!("global slot {idx} is undefined"),
                        ));
                    };
                    self.push(value, ip)?;
                    self.advance_ip(3)?;
                }
                Opcode::GetLocal => {
                    let idx = self.read_u8_operand(ip)?;
                    let base = self.current_frame_required(ip)?.base_pointer;
                    let slot = base + idx;
                    let Some(value) = self.stack.get(slot).cloned() else {
                        return Err(self.runtime_error(
                            ip,
                            RuntimeErrorType::UnsupportedOperation,
                            format!("local slot out of bounds: {idx}"),
                        ));
                    };
                    self.push(value, ip)?;
                    self.advance_ip(2)?;
                }
                Opcode::SetLocal => {
                    let idx = self.read_u8_operand(ip)?;
                    let value = self.pop(ip)?;
                    let base = self.current_frame_required(ip)?.base_pointer;
                    let slot = base + idx;
                    if slot >= self.stack.len() {
                        return Err(self.runtime_error(
                            ip,
                            RuntimeErrorType::UnsupportedOperation,
                            format!("local slot out of bounds: {idx}"),
                        ));
                    }
                    self.stack[slot] = value;
                    self.advance_ip(2)?;
                }
                Opcode::GetBuiltin => {
                    let idx = self.read_u8_operand(ip)?;
                    let Some(name) = builtin_name_at(idx) else {
                        return Err(self.runtime_error(
                            ip,
                            RuntimeErrorType::UnsupportedOperation,
                            format!("unknown builtin index: {idx}"),
                        ));
                    };
                    self.push(
                        Object::Builtin(crate::object::BuiltinObject {
                            name: name.to_string(),
                        })
                        .rc(),
                        ip,
                    )?;
                    self.advance_ip(2)?;
                }
                Opcode::GetFree => {
                    let idx = self.read_u8_operand(ip)?;
                    let Some(value) = self
                        .current_frame_required(ip)?
                        .closure
                        .free
                        .get(idx)
                        .cloned()
                    else {
                        return Err(self.runtime_error(
                            ip,
                            RuntimeErrorType::UnsupportedOperation,
                            format!("free variable out of bounds: {idx}"),
                        ));
                    };
                    self.push(value, ip)?;
                    self.advance_ip(2)?;
                }
                Opcode::CurrentClosure => {
                    let closure = Rc::clone(&self.current_frame_required(ip)?.closure);
                    self.push(Object::Closure(closure).rc(), ip)?;
                    self.advance_ip(1)?;
                }
                Opcode::Closure => {
                    let const_idx = self.read_u16_operand(ip)?;
                    let free_count = self.read_u8_at(ip + 3, ip)?;
                    let Some(constant) = self.chunk.constants.get(const_idx).cloned() else {
                        return Err(self.runtime_error(
                            ip,
                            RuntimeErrorType::UnsupportedOperation,
                            format!("constant index out of bounds: {const_idx}"),
                        ));
                    };
                    let function = match constant.as_ref() {
                        Object::CompiledFunction(f) => Rc::clone(f),
                        other => {
                            return Err(self.runtime_error(
                                ip,
                                RuntimeErrorType::TypeMismatch,
                                format!(
                                    "closure constant is not a compiled function: {}",
                                    other.type_name()
                                ),
                            ));
                        }
                    };

                    if self.stack.len() < free_count {
                        return Err(self.runtime_error(
                            ip,
                            RuntimeErrorType::UnsupportedOperation,
                            "stack underflow while capturing free variables",
                        ));
                    }
                    let start = self.stack.len() - free_count;
                    let free = self.stack[start..].to_vec();
                    self.stack.truncate(start);

                    let closure = Rc::new(ClosureObject { function, free });
                    self.push(Object::Closure(closure).rc(), ip)?;
                    self.advance_ip(4)?;
                }
                Opcode::Call => {
                    let argc = self.read_u8_operand(ip)?;
                    self.advance_ip(2)?;
                    self.exec_call(argc, ip)?;
                }
                Opcode::ReturnValue => {
                    let value = self.pop(ip)?;
                    if let Some(final_value) = self.return_from_frame(value)? {
                        return Ok(final_value);
                    }
                }
                Opcode::Return => {
                    if let Some(final_value) = self.return_from_frame(Object::Null.rc())? {
                        return Ok(final_value);
                    }
                }
                Opcode::Array => {
                    let count = self.read_u16_operand(ip)?;
                    if self.stack.len() < count {
                        return Err(self.runtime_error(
                            ip,
                            RuntimeErrorType::UnsupportedOperation,
                            "stack underflow while building array",
                        ));
                    }
                    let start = self.stack.len() - count;
                    let items = self.stack[start..].to_vec();
                    self.stack.truncate(start);
                    self.push(Object::Array(items).rc(), ip)?;
                    self.advance_ip(3)?;
                }
                Opcode::Hash => {
                    let pair_count = self.read_u16_operand(ip)?;
                    let value_count = pair_count * 2;
                    if self.stack.len() < value_count {
                        return Err(self.runtime_error(
                            ip,
                            RuntimeErrorType::UnsupportedOperation,
                            "stack underflow while building hash",
                        ));
                    }
                    let start = self.stack.len() - value_count;
                    let values = self.stack[start..].to_vec();
                    self.stack.truncate(start);

                    let mut pairs = Vec::with_capacity(pair_count);
                    for i in 0..pair_count {
                        let key = values[i * 2].clone();
                        let value = values[i * 2 + 1].clone();
                        if key.as_ref().hash_key().is_none() {
                            return Err(self.runtime_error(
                                ip,
                                RuntimeErrorType::Unhashable,
                                format!("unusable as hash key: {}", key.as_ref().type_name()),
                            ));
                        }
                        pairs.push((key, value));
                    }
                    self.push(Object::Hash(pairs).rc(), ip)?;
                    self.advance_ip(3)?;
                }
                Opcode::Index => {
                    let index = self.pop(ip)?;
                    let left = self.pop(ip)?;
                    let out = self.exec_index(left, index, ip)?;
                    self.push(out, ip)?;
                    self.advance_ip(1)?;
                }
                Opcode::InvalidBreak => {
                    return Err(self.runtime_error(
                        ip,
                        RuntimeErrorType::InvalidControlFlow,
                        "break used outside of loop",
                    ));
                }
                Opcode::InvalidContinue => {
                    return Err(self.runtime_error(
                        ip,
                        RuntimeErrorType::InvalidControlFlow,
                        "continue used outside of loop",
                    ));
                }
                Opcode::Nop => {
                    return Err(self.runtime_error(
                        ip,
                        RuntimeErrorType::UnsupportedOperation,
                        "opcode not implemented in step 17: Nop",
                    ));
                }
            }
        }

        Ok(Object::Null.rc())
    }

    pub fn last_popped(&self) -> Option<ObjectRef> {
        self.last_popped.clone()
    }

    pub fn globals(&self) -> &[ObjectRef] {
        &self.globals
    }

    pub fn output(&self) -> &[String] {
        &self.output
    }

    pub fn take_output(&mut self) -> Vec<String> {
        std::mem::take(&mut self.output)
    }

    fn exec_call(&mut self, argc: usize, ip: usize) -> Result<(), RuntimeError> {
        if self.stack.len() < argc + 1 {
            return Err(self.runtime_error(
                ip,
                RuntimeErrorType::UnsupportedOperation,
                "stack underflow while preparing call",
            ));
        }
        let callee_index = self.stack.len() - 1 - argc;
        let callee = self.stack[callee_index].clone();
        match callee.as_ref() {
            Object::Closure(closure) => self.call_closure(Rc::clone(closure), argc, ip),
            Object::Builtin(builtin) => self.call_builtin(&builtin.name, argc, callee_index, ip),
            other => Err(self.runtime_error(
                ip,
                RuntimeErrorType::NotCallable,
                format!("object is not callable: {}", other.type_name()),
            )),
        }
    }

    fn call_closure(
        &mut self,
        closure: Rc<ClosureObject>,
        argc: usize,
        ip: usize,
    ) -> Result<(), RuntimeError> {
        let expected = closure.function.num_params;
        if argc != expected {
            return Err(self.runtime_error(
                ip,
                RuntimeErrorType::WrongArgumentCount,
                format!(
                    "{} expected {} argument(s), got {}",
                    closure.function.name.as_deref().unwrap_or("<anonymous>"),
                    expected,
                    argc
                ),
            ));
        }

        let callee_index = self.stack.len() - 1 - argc;
        let base_pointer = callee_index + 1;
        let required = base_pointer + closure.function.num_locals;
        while self.stack.len() < required {
            self.stack.push(Object::Null.rc());
        }
        let call_pos = self.current_position(ip);
        self.push_frame(Frame::new(closure, base_pointer, call_pos, argc));
        Ok(())
    }

    fn call_builtin(
        &mut self,
        name: &str,
        argc: usize,
        callee_index: usize,
        ip: usize,
    ) -> Result<(), RuntimeError> {
        let args_start = callee_index + 1;
        let args_end = args_start + argc;
        let args = self.stack[args_start..args_end].to_vec();
        let result = execute_builtin(name, &args, &mut self.output)
            .map_err(|err| self.runtime_error(ip, err.error_type, err.message))?;
        self.stack.truncate(callee_index);
        self.push(result, ip)
    }

    fn return_from_frame(&mut self, value: ObjectRef) -> Result<Option<ObjectRef>, RuntimeError> {
        let Some(frame) = self.pop_frame() else {
            return Err(RuntimeError::new(
                RuntimeErrorType::UnsupportedOperation,
                "frame stack underflow on return",
                Position::default(),
            ));
        };

        if self.frames.is_empty() {
            return Ok(Some(value));
        }

        let truncate_to = frame.base_pointer.saturating_sub(1);
        self.stack.truncate(truncate_to);
        let caller_ip = self.current_frame_required(0)?.ip;
        self.push(value, caller_ip)?;
        Ok(None)
    }

    fn exec_index(
        &self,
        left: ObjectRef,
        index: ObjectRef,
        ip: usize,
    ) -> Result<ObjectRef, RuntimeError> {
        match left.as_ref() {
            Object::Array(values) => match index.as_ref() {
                Object::Integer(i) => {
                    if *i < 0 {
                        Ok(Object::Null.rc())
                    } else {
                        Ok(values
                            .get(*i as usize)
                            .cloned()
                            .unwrap_or_else(|| Object::Null.rc()))
                    }
                }
                other => Err(self.runtime_error(
                    ip,
                    RuntimeErrorType::InvalidIndex,
                    format!("array index must be INTEGER, got {}", other.type_name()),
                )),
            },
            Object::Hash(pairs) => {
                let Some(target_key) = index.as_ref().hash_key() else {
                    return Err(self.runtime_error(
                        ip,
                        RuntimeErrorType::Unhashable,
                        format!("unusable as hash key: {}", index.as_ref().type_name()),
                    ));
                };

                for (key, value) in pairs.iter().rev() {
                    if key.as_ref().hash_key() == Some(target_key.clone()) {
                        return Ok(value.clone());
                    }
                }
                Ok(Object::Null.rc())
            }
            other => Err(self.runtime_error(
                ip,
                RuntimeErrorType::InvalidIndex,
                format!("index operator not supported: {}", other.type_name()),
            )),
        }
    }

    fn push(&mut self, obj: ObjectRef, ip: usize) -> Result<(), RuntimeError> {
        if self.stack.len() == usize::MAX {
            return Err(self.runtime_error(
                ip,
                RuntimeErrorType::UnsupportedOperation,
                "stack overflow",
            ));
        }
        self.stack.push(obj);
        Ok(())
    }

    fn pop(&mut self, ip: usize) -> Result<ObjectRef, RuntimeError> {
        let value = self.stack.pop().ok_or_else(|| {
            self.runtime_error(
                ip,
                RuntimeErrorType::UnsupportedOperation,
                "stack underflow",
            )
        })?;
        self.last_popped = Some(value.clone());
        Ok(value)
    }

    fn peek(&self, ip: usize) -> Result<&ObjectRef, RuntimeError> {
        self.stack.last().ok_or_else(|| {
            self.runtime_error(
                ip,
                RuntimeErrorType::UnsupportedOperation,
                "stack underflow",
            )
        })
    }

    fn read_u8_at(&self, byte_index: usize, ip: usize) -> Result<usize, RuntimeError> {
        let value = self.current_instructions().get(byte_index).ok_or_else(|| {
            self.runtime_error(
                ip,
                RuntimeErrorType::UnsupportedOperation,
                format!("truncated instruction at offset {ip}"),
            )
        })?;
        Ok(*value as usize)
    }

    fn read_u8_operand(&self, ip: usize) -> Result<usize, RuntimeError> {
        self.read_u8_at(ip + 1, ip)
    }

    fn read_u16_operand(&self, ip: usize) -> Result<usize, RuntimeError> {
        let hi = self.current_instructions().get(ip + 1).ok_or_else(|| {
            self.runtime_error(
                ip,
                RuntimeErrorType::UnsupportedOperation,
                format!("truncated instruction at offset {ip}"),
            )
        })?;
        let lo = self.current_instructions().get(ip + 2).ok_or_else(|| {
            self.runtime_error(
                ip,
                RuntimeErrorType::UnsupportedOperation,
                format!("truncated instruction at offset {ip}"),
            )
        })?;
        Ok(u16::from_be_bytes([*hi, *lo]) as usize)
    }

    fn ensure_jump_target(&self, ip: usize, target: usize) -> Result<(), RuntimeError> {
        if target > self.current_instructions().len() {
            return Err(self.runtime_error(
                ip,
                RuntimeErrorType::UnsupportedOperation,
                format!(
                    "jump target out of bounds: {target} (len {})",
                    self.current_instructions().len()
                ),
            ));
        }
        Ok(())
    }

    fn exec_binary_arithmetic(&mut self, op: Opcode, ip: usize) -> Result<(), RuntimeError> {
        let right = self.pop(ip)?;
        let left = self.pop(ip)?;

        let result = match (left.as_ref(), right.as_ref(), op) {
            (Object::Integer(a), Object::Integer(b), Opcode::Add) => Object::Integer(a + b).rc(),
            (Object::Integer(a), Object::Integer(b), Opcode::Sub) => Object::Integer(a - b).rc(),
            (Object::Integer(a), Object::Integer(b), Opcode::Mul) => Object::Integer(a * b).rc(),
            (Object::Integer(_), Object::Integer(0), Opcode::Div) => {
                return Err(self.runtime_error(
                    ip,
                    RuntimeErrorType::DivisionByZero,
                    "division by zero",
                ));
            }
            (Object::Integer(a), Object::Integer(b), Opcode::Div) => Object::Integer(a / b).rc(),
            (Object::String(a), Object::String(b), Opcode::Add) => {
                Object::String(format!("{a}{b}")).rc()
            }
            (Object::String(_), Object::String(_), _) => {
                return Err(self.runtime_error(
                    ip,
                    RuntimeErrorType::UnsupportedOperation,
                    format!(
                        "unsupported string operation: {}",
                        lookup_definition(op).name
                    ),
                ));
            }
            (l, r, _) => {
                return Err(self.runtime_error(
                    ip,
                    RuntimeErrorType::TypeMismatch,
                    format!(
                        "unsupported operand types for {}: {} and {}",
                        lookup_definition(op).name,
                        l.type_name(),
                        r.type_name()
                    ),
                ));
            }
        };

        self.push(result, ip)
    }

    fn exec_comparison(&mut self, op: Opcode, ip: usize) -> Result<(), RuntimeError> {
        let right = self.pop(ip)?;
        let left = self.pop(ip)?;

        let value = match (left.as_ref(), right.as_ref()) {
            (Object::Integer(a), Object::Integer(b)) => match op {
                Opcode::Eq => a == b,
                Opcode::Ne => a != b,
                Opcode::Lt => a < b,
                Opcode::Gt => a > b,
                Opcode::Le => a <= b,
                Opcode::Ge => a >= b,
                _ => unreachable!("comparison opcode already filtered"),
            },
            (Object::Boolean(a), Object::Boolean(b)) => match op {
                Opcode::Eq => a == b,
                Opcode::Ne => a != b,
                _ => {
                    return Err(self.runtime_error(
                        ip,
                        RuntimeErrorType::TypeMismatch,
                        format!(
                            "unsupported operand types for {}: BOOLEAN and BOOLEAN",
                            lookup_definition(op).name
                        ),
                    ));
                }
            },
            (Object::Null, Object::Null) => match op {
                Opcode::Eq => true,
                Opcode::Ne => false,
                _ => {
                    return Err(self.runtime_error(
                        ip,
                        RuntimeErrorType::TypeMismatch,
                        format!(
                            "unsupported operand types for {}: NULL and NULL",
                            lookup_definition(op).name
                        ),
                    ));
                }
            },
            (Object::String(a), Object::String(b)) => match op {
                Opcode::Eq => a == b,
                Opcode::Ne => a != b,
                Opcode::Lt | Opcode::Gt | Opcode::Le | Opcode::Ge => {
                    return Err(self.runtime_error(
                        ip,
                        RuntimeErrorType::UnsupportedOperation,
                        format!(
                            "unsupported string operation: {}",
                            lookup_definition(op).name
                        ),
                    ));
                }
                _ => unreachable!("comparison opcode already filtered"),
            },
            (l, r) => {
                return Err(self.runtime_error(
                    ip,
                    RuntimeErrorType::TypeMismatch,
                    format!(
                        "unsupported operand types for {}: {} and {}",
                        lookup_definition(op).name,
                        l.type_name(),
                        r.type_name()
                    ),
                ));
            }
        };

        self.push(Object::Boolean(value).rc(), ip)
    }

    fn current_frame(&self) -> Option<&Frame> {
        self.frames.last()
    }

    fn current_frame_required(&self, ip: usize) -> Result<&Frame, RuntimeError> {
        self.current_frame().ok_or_else(|| {
            self.runtime_error(
                ip,
                RuntimeErrorType::UnsupportedOperation,
                "frame stack underflow",
            )
        })
    }

    fn current_frame_mut(&mut self) -> Option<&mut Frame> {
        self.frames.last_mut()
    }

    fn current_instructions(&self) -> &[u8] {
        &self
            .current_frame()
            .expect("frame stack should never be empty here")
            .closure
            .function
            .instructions
    }

    fn advance_ip(&mut self, delta: usize) -> Result<(), RuntimeError> {
        let frame = self.current_frame_mut().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorType::UnsupportedOperation,
                "frame stack underflow",
                Position::default(),
            )
        })?;
        frame.ip += delta;
        Ok(())
    }

    fn set_ip(&mut self, ip: usize) -> Result<(), RuntimeError> {
        let frame = self.current_frame_mut().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorType::UnsupportedOperation,
                "frame stack underflow",
                Position::default(),
            )
        })?;
        frame.ip = ip;
        Ok(())
    }

    fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    fn pop_frame(&mut self) -> Option<Frame> {
        self.frames.pop()
    }

    fn current_position(&self, ip: usize) -> Position {
        let Some(frame) = self.current_frame() else {
            return Position::default();
        };
        frame
            .closure
            .function
            .positions
            .iter()
            .take_while(|(offset, _)| *offset <= ip)
            .last()
            .map(|(_, pos)| *pos)
            .unwrap_or_default()
    }

    fn runtime_error(
        &self,
        ip: usize,
        error_type: RuntimeErrorType,
        message: impl Into<String>,
    ) -> RuntimeError {
        let pos = self.current_position(ip);
        let stack = self.build_stack_trace(ip);
        RuntimeError::new(error_type, message, pos).with_stack(stack)
    }

    fn build_stack_trace(&self, current_ip: usize) -> Vec<StackFrameInfo> {
        let mut out = Vec::new();
        for (idx, frame) in self.frames.iter().enumerate().rev() {
            let name = frame
                .closure
                .function
                .name
                .clone()
                .unwrap_or_else(|| "<anonymous>".to_string());
            let pos = if idx == self.frames.len() - 1 {
                frame
                    .closure
                    .function
                    .positions
                    .iter()
                    .take_while(|(offset, _)| *offset <= current_ip)
                    .last()
                    .map(|(_, pos)| *pos)
                    .unwrap_or_default()
            } else {
                frame.call_site_pos
            };
            out.push(StackFrameInfo::new(name, pos).with_arg_count(frame.arg_count));
        }
        out
    }
}
