use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::ast::{BlockStatement, Expression, Program, Statement};
use crate::bytecode::{make, BytecodeError, Chunk, Opcode};
use crate::object::Object;
use crate::position::Position;
use crate::symbol_table::{define_builtins, Symbol, SymbolScope, SymbolTable};

/// Deterministic compile-time error for unsupported or invalid compiler input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    pub message: String,
    pub pos: Option<Position>,
}

impl CompileError {
    pub fn new(message: impl Into<String>, pos: Option<Position>) -> Self {
        Self {
            message: message.into(),
            pos,
        }
    }

    fn unsupported_expression(name: &str, pos: Position) -> Self {
        Self::new(
            format!("unsupported expression in step 13: {name}"),
            Some(pos),
        )
    }

    fn unresolved_identifier(name: &str, pos: Position) -> Self {
        Self::new(format!("unresolved identifier: {name}"), Some(pos))
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.pos {
            Some(pos) => write!(f, "{pos}: {}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

/// Phase-1 compiler for basic expressions and let statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EmittedInstruction {
    opcode: Opcode,
    offset: usize,
}

#[derive(Debug, Clone)]
struct LoopContext {
    continue_target: usize,
    break_jumps: Vec<usize>,
    #[allow(dead_code)]
    loop_pos: Position,
}

/// Phase-1 compiler for basic expressions and let statements.
#[derive(Debug)]
pub struct Compiler {
    chunk: Chunk,
    symbol_table: crate::symbol_table::SymbolTableRef,
    last_instruction: Option<EmittedInstruction>,
    previous_instruction: Option<EmittedInstruction>,
    loop_stack: Vec<LoopContext>,
}

impl Compiler {
    pub fn new() -> Self {
        let mut root = SymbolTable::new();
        define_builtins(&mut root);

        Self {
            chunk: Chunk::new(),
            symbol_table: root.into_ref(),
            last_instruction: None,
            previous_instruction: None,
            loop_stack: Vec::new(),
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<(), CompileError> {
        for stmt in &program.statements {
            self.compile_statement(stmt)?;
        }

        let terminal_pos = program
            .statements
            .last()
            .map(Statement::pos)
            .unwrap_or_default();

        let ends_with_expression = matches!(
            program.statements.last(),
            Some(Statement::Expression { .. })
        );

        if ends_with_expression && self.last_instruction_is(Opcode::Pop) {
            self.replace_last_pop_with_return_value(terminal_pos)?;
        } else if !self.last_instruction_is(Opcode::ReturnValue)
            && !self.last_instruction_is(Opcode::Return)
        {
            self.emit(Opcode::Return, &[], terminal_pos)?;
        }

        Ok(())
    }

    pub fn compile_statement(&mut self, stmt: &Statement) -> Result<(), CompileError> {
        match stmt {
            Statement::Let { name, value, pos } => {
                self.compile_expression(value)?;
                let symbol = self.symbol_table.borrow_mut().define(name.value.clone());

                match symbol.scope {
                    SymbolScope::Global => {
                        self.emit(Opcode::SetGlobal, &[symbol.index], *pos)?;
                    }
                    SymbolScope::Local => {
                        self.emit(Opcode::SetLocal, &[symbol.index], *pos)?;
                    }
                    _ => {
                        return Err(CompileError::new(
                            format!(
                                "invalid symbol scope for let binding '{}': {}",
                                name.value, symbol.scope
                            ),
                            Some(*pos),
                        ));
                    }
                }
            }
            Statement::Expression { expression, pos } => {
                self.compile_expression(expression)?;
                self.emit(Opcode::Pop, &[], *pos)?;
            }
            Statement::Return { value, pos } => {
                self.compile_expression(value)?;
                self.emit(Opcode::ReturnValue, &[], *pos)?;
            }
            Statement::While {
                condition,
                body,
                pos,
            } => {
                let loop_start = self.current_offset();
                self.loop_stack.push(LoopContext {
                    continue_target: loop_start,
                    break_jumps: Vec::new(),
                    loop_pos: *pos,
                });

                self.compile_expression(condition)?;
                let false_jump = self.emit_jump(Opcode::JumpIfFalse, *pos)?;
                self.emit_pop(*pos)?;

                self.compile_block(body)?;
                self.emit(Opcode::Jump, &[loop_start], *pos)?;

                let cond_false_label = self.current_offset();
                self.patch_jump(false_jump, cond_false_label)?;
                self.emit_pop(*pos)?;
                let loop_end = self.current_offset();

                let loop_ctx = self.loop_stack.pop().ok_or_else(|| {
                    CompileError::new("while loop context stack underflow", Some(*pos))
                })?;
                for break_jump in loop_ctx.break_jumps {
                    self.patch_jump(break_jump, loop_end)?;
                }
            }
            Statement::Break { pos } => {
                if self.loop_stack.is_empty() {
                    // TODO(step-17): VM will translate this opcode into INVALID_CONTROL_FLOW.
                    self.emit(Opcode::InvalidBreak, &[], *pos)?;
                } else {
                    let break_jump = self.emit_jump(Opcode::Jump, *pos)?;
                    if let Some(loop_ctx) = self.current_loop_mut() {
                        loop_ctx.break_jumps.push(break_jump);
                    } else {
                        return Err(CompileError::new(
                            "break compilation lost loop context",
                            Some(*pos),
                        ));
                    }
                }
            }
            Statement::Continue { pos } => {
                if let Some(loop_ctx) = self.current_loop() {
                    self.emit(Opcode::Jump, &[loop_ctx.continue_target], *pos)?;
                } else {
                    // TODO(step-17): VM will translate this opcode into INVALID_CONTROL_FLOW.
                    self.emit(Opcode::InvalidContinue, &[], *pos)?;
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn compile_block(&mut self, block: &BlockStatement) -> Result<(), CompileError> {
        // TODO(step-14): function-body compilation will reuse statement-context block compilation.
        for stmt in &block.statements {
            self.compile_statement(stmt)?;
        }
        Ok(())
    }

    fn compile_block_expression_value(
        &mut self,
        block: &BlockStatement,
        owner_pos: Position,
    ) -> Result<(), CompileError> {
        // TODO(step-14): function-body expression mode can share this branch-value shaping.
        self.compile_block(block)?;

        if self.last_instruction_is(Opcode::Pop) {
            self.remove_last_pop()?;
        } else if !self.last_instruction_is(Opcode::ReturnValue)
            && !self.last_instruction_is(Opcode::Return)
        {
            self.emit(Opcode::Null, &[], owner_pos)?;
        }

        Ok(())
    }

    pub fn compile_expression(&mut self, expr: &Expression) -> Result<(), CompileError> {
        match expr {
            Expression::IntegerLiteral { value, pos, .. } => {
                let idx = self.add_constant(Object::Integer(*value), *pos);
                self.emit(Opcode::Constant, &[idx], *pos)?;
            }
            Expression::BooleanLiteral { value, pos } => {
                if *value {
                    self.emit(Opcode::True, &[], *pos)?;
                } else {
                    self.emit(Opcode::False, &[], *pos)?;
                }
            }
            Expression::StringLiteral { value, pos } => {
                let idx = self.add_constant(Object::String(value.clone()), *pos);
                self.emit(Opcode::Constant, &[idx], *pos)?;
            }
            Expression::Identifier { value, pos } => {
                let symbol = self.symbol_table.borrow_mut().resolve(value);
                let Some(symbol) = symbol else {
                    // TODO(step-17): unresolved identifiers should align with runtime UNKNOWN_IDENTIFIER flow.
                    return Err(CompileError::unresolved_identifier(value, *pos));
                };
                self.emit_for_symbol_load(&symbol, *pos)?;
            }
            Expression::Prefix {
                operator,
                right,
                pos,
            } => {
                self.compile_expression(right)?;
                match operator.as_str() {
                    "!" => {
                        self.emit(Opcode::Bang, &[], *pos)?;
                    }
                    "-" => {
                        self.emit(Opcode::Neg, &[], *pos)?;
                    }
                    _ => {
                        return Err(CompileError::new(
                            format!("unsupported prefix operator in step 13: {operator}"),
                            Some(*pos),
                        ));
                    }
                }
            }
            Expression::Infix {
                left,
                operator,
                right,
                pos,
            } => {
                match operator.as_str() {
                    "&&" => {
                        // TODO(step-12): reuse jump patching helpers for control-flow expressions/statements.
                        self.compile_expression(left)?;
                        let false_jump = self.emit_jump(Opcode::JumpIfFalse, *pos)?;
                        self.emit_pop(*pos)?;

                        self.compile_expression(right)?;
                        self.emit_bool_normalize(*pos)?;
                        let end_jump = self.emit_jump(Opcode::Jump, *pos)?;

                        let false_branch = self.current_offset();
                        self.patch_jump(false_jump, false_branch)?;
                        self.emit_pop(*pos)?;
                        self.emit(Opcode::False, &[], *pos)?;

                        let end_offset = self.current_offset();
                        self.patch_jump(end_jump, end_offset)?;
                        return Ok(());
                    }
                    "||" => {
                        // TODO(step-12): reuse jump patching helpers for control-flow expressions/statements.
                        self.compile_expression(left)?;
                        let rhs_jump = self.emit_jump(Opcode::JumpIfFalse, *pos)?;
                        self.emit_pop(*pos)?;
                        self.emit(Opcode::True, &[], *pos)?;
                        let end_jump = self.emit_jump(Opcode::Jump, *pos)?;

                        let rhs_offset = self.current_offset();
                        self.patch_jump(rhs_jump, rhs_offset)?;
                        self.emit_pop(*pos)?;
                        self.compile_expression(right)?;
                        self.emit_bool_normalize(*pos)?;

                        let end_offset = self.current_offset();
                        self.patch_jump(end_jump, end_offset)?;
                        return Ok(());
                    }
                    _ => {}
                }

                self.compile_expression(left)?;
                self.compile_expression(right)?;

                let opcode = match operator.as_str() {
                    "+" => Opcode::Add,
                    "-" => Opcode::Sub,
                    "*" => Opcode::Mul,
                    "/" => Opcode::Div,
                    "==" => Opcode::Eq,
                    "!=" => Opcode::Ne,
                    "<" => Opcode::Lt,
                    ">" => Opcode::Gt,
                    "<=" => Opcode::Le,
                    ">=" => Opcode::Ge,
                    _ => {
                        return Err(CompileError::new(
                            format!("unsupported infix operator in step 13: {operator}"),
                            Some(*pos),
                        ));
                    }
                };
                self.emit(opcode, &[], *pos)?;
            }
            Expression::If {
                condition,
                consequence,
                alternative,
                pos,
            } => {
                self.compile_expression(condition)?;
                let false_jump = self.emit_jump(Opcode::JumpIfFalse, *pos)?;
                self.emit_pop(*pos)?;

                self.compile_block_expression_value(consequence, *pos)?;
                let end_jump = self.emit_jump(Opcode::Jump, *pos)?;

                let false_branch = self.current_offset();
                self.patch_jump(false_jump, false_branch)?;
                self.emit_pop(*pos)?;

                match alternative {
                    Some(block) => self.compile_block_expression_value(block, *pos)?,
                    None => {
                        self.emit(Opcode::Null, &[], *pos)?;
                    }
                }

                let end_offset = self.current_offset();
                self.patch_jump(end_jump, end_offset)?;
            }
            Expression::FunctionLiteral { pos, .. } => {
                // TODO(step-14): compile function literals and closures.
                return Err(CompileError::unsupported_expression(
                    "FunctionLiteral",
                    *pos,
                ));
            }
            Expression::Call { pos, .. } => {
                // TODO(step-14): compile function calls.
                return Err(CompileError::unsupported_expression("Call", *pos));
            }
            Expression::ArrayLiteral { pos, .. } => {
                // TODO(step-15): compile array literals.
                return Err(CompileError::unsupported_expression("ArrayLiteral", *pos));
            }
            Expression::HashLiteral { pos, .. } => {
                // TODO(step-15): compile hash literals.
                return Err(CompileError::unsupported_expression("HashLiteral", *pos));
            }
            Expression::Index { pos, .. } => {
                // TODO(step-15): compile index expressions.
                return Err(CompileError::unsupported_expression("Index", *pos));
            }
        }

        Ok(())
    }

    pub fn compile(&mut self, program: &Program) -> Result<(), CompileError> {
        self.compile_program(program)
    }

    pub fn bytecode(&self) -> &Chunk {
        &self.chunk
    }

    pub fn into_bytecode(self) -> Chunk {
        self.chunk
    }

    fn emit(
        &mut self,
        op: Opcode,
        operands: &[usize],
        pos: Position,
    ) -> Result<usize, CompileError> {
        let bytes = make(op, operands).map_err(|err| self.bytecode_error(op, pos, err))?;
        let offset = self.chunk.push_bytes(&bytes);
        self.chunk.record_pos(offset, pos);
        self.set_last_instruction(op, offset);
        Ok(offset)
    }

    fn current_offset(&self) -> usize {
        self.chunk.instructions.len()
    }

    fn emit_jump(&mut self, op: Opcode, pos: Position) -> Result<usize, CompileError> {
        self.emit(op, &[0], pos)
    }

    fn patch_jump(&mut self, jump_offset: usize, target_offset: usize) -> Result<(), CompileError> {
        if jump_offset >= self.chunk.instructions.len() {
            return Err(CompileError::new(
                format!(
                    "invalid jump patch offset {} for instruction length {}",
                    jump_offset,
                    self.chunk.instructions.len()
                ),
                None,
            ));
        }

        let opcode_byte = self.chunk.instructions[jump_offset];
        let Some(opcode) = Opcode::from_byte(opcode_byte) else {
            return Err(CompileError::new(
                format!("cannot patch unknown opcode byte {opcode_byte} at {jump_offset}"),
                None,
            ));
        };

        if !matches!(opcode, Opcode::Jump | Opcode::JumpIfFalse) {
            return Err(CompileError::new(
                format!(
                    "cannot patch non-jump opcode {} at {}",
                    crate::bytecode::lookup_definition(opcode).name,
                    jump_offset
                ),
                None,
            ));
        }

        let patched = make(opcode, &[target_offset]).map_err(|err| {
            CompileError::new(
                format!(
                    "failed to patch {} at {}: {err}",
                    crate::bytecode::lookup_definition(opcode).name,
                    jump_offset
                ),
                None,
            )
        })?;

        let end = jump_offset + patched.len();
        if end > self.chunk.instructions.len() {
            return Err(CompileError::new(
                format!(
                    "patched jump overflows instruction buffer: {}..{} of {}",
                    jump_offset,
                    end,
                    self.chunk.instructions.len()
                ),
                None,
            ));
        }

        self.chunk.instructions[jump_offset..end].copy_from_slice(&patched);
        Ok(())
    }

    fn replace_instruction(&mut self, offset: usize, bytes: &[u8]) -> Result<(), CompileError> {
        let end = offset + bytes.len();
        if end > self.chunk.instructions.len() {
            return Err(CompileError::new(
                format!(
                    "replacement instruction out of bounds: {}..{} of {}",
                    offset,
                    end,
                    self.chunk.instructions.len()
                ),
                None,
            ));
        }
        self.chunk.instructions[offset..end].copy_from_slice(bytes);
        Ok(())
    }

    fn remove_last_instruction(&mut self) -> Result<(), CompileError> {
        let Some(last) = self.last_instruction else {
            return Err(CompileError::new(
                "cannot remove last instruction: no instructions emitted",
                None,
            ));
        };

        self.chunk.instructions.truncate(last.offset);
        self.record_last_instruction_from_tail()?;
        Ok(())
    }

    fn remove_last_pop(&mut self) -> Result<(), CompileError> {
        if !self.last_instruction_is(Opcode::Pop) {
            return Err(CompileError::new(
                "cannot remove last Pop: last instruction is not Pop",
                None,
            ));
        }
        self.remove_last_instruction()
    }

    fn set_last_instruction(&mut self, opcode: Opcode, offset: usize) {
        self.previous_instruction = self.last_instruction;
        self.last_instruction = Some(EmittedInstruction { opcode, offset });
    }

    fn record_last_instruction_from_tail(&mut self) -> Result<(), CompileError> {
        let mut decoded = Vec::new();
        let mut offset = 0;

        while offset < self.chunk.instructions.len() {
            let byte = self.chunk.instructions[offset];
            let Some(opcode) = Opcode::from_byte(byte) else {
                return Err(CompileError::new(
                    format!("unknown opcode byte {byte} at offset {offset}"),
                    None,
                ));
            };
            let def = crate::bytecode::lookup_definition(opcode);
            let operand_len: usize = def.operand_widths.iter().sum();
            let end = offset + 1 + operand_len;
            if end > self.chunk.instructions.len() {
                return Err(CompileError::new(
                    format!(
                        "truncated instruction while rebuilding instruction tracking at offset {offset}"
                    ),
                    None,
                ));
            }

            decoded.push(EmittedInstruction { opcode, offset });
            offset = end;
        }

        self.last_instruction = decoded.last().copied();
        self.previous_instruction = if decoded.len() > 1 {
            Some(decoded[decoded.len() - 2])
        } else {
            None
        };
        Ok(())
    }

    fn last_instruction_is(&self, opcode: Opcode) -> bool {
        self.last_instruction
            .map(|ins| ins.opcode == opcode)
            .unwrap_or(false)
    }

    fn replace_last_pop_with_return_value(&mut self, pos: Position) -> Result<(), CompileError> {
        let Some(last) = self.last_instruction else {
            return Err(CompileError::new(
                "cannot replace last Pop: no instructions emitted",
                Some(pos),
            ));
        };

        if last.opcode != Opcode::Pop {
            return Err(CompileError::new(
                format!(
                    "cannot replace last instruction {} with ReturnValue",
                    crate::bytecode::lookup_definition(last.opcode).name
                ),
                Some(pos),
            ));
        }

        let bytes = make(Opcode::ReturnValue, &[])
            .map_err(|err| self.bytecode_error(Opcode::ReturnValue, pos, err))?;
        self.replace_instruction(last.offset, &bytes)?;
        self.last_instruction = Some(EmittedInstruction {
            opcode: Opcode::ReturnValue,
            offset: last.offset,
        });
        Ok(())
    }

    fn emit_pop(&mut self, pos: Position) -> Result<usize, CompileError> {
        self.emit(Opcode::Pop, &[], pos)
    }

    fn emit_bool_normalize(&mut self, pos: Position) -> Result<(), CompileError> {
        self.emit(Opcode::Bang, &[], pos)?;
        self.emit(Opcode::Bang, &[], pos)?;
        Ok(())
    }

    fn add_constant(&mut self, obj: Object, _pos: Position) -> usize {
        self.chunk.add_constant(obj.rc())
    }

    fn emit_for_symbol_load(&mut self, symbol: &Symbol, pos: Position) -> Result<(), CompileError> {
        match symbol.scope {
            SymbolScope::Global => {
                self.emit(Opcode::GetGlobal, &[symbol.index], pos)?;
            }
            SymbolScope::Local => {
                self.emit(Opcode::GetLocal, &[symbol.index], pos)?;
            }
            SymbolScope::Builtin => {
                self.emit(Opcode::GetBuiltin, &[symbol.index], pos)?;
            }
            SymbolScope::Free => {
                self.emit(Opcode::GetFree, &[symbol.index], pos)?;
            }
            SymbolScope::Function => {
                return Err(CompileError::new(
                    format!(
                        "unsupported function symbol load in step 13: {}",
                        symbol.name
                    ),
                    Some(pos),
                ));
            }
        }
        Ok(())
    }

    fn bytecode_error(&self, op: Opcode, pos: Position, err: BytecodeError) -> CompileError {
        CompileError::new(
            format!(
                "failed to emit {}: {err}",
                crate::bytecode::lookup_definition(op).name
            ),
            Some(pos),
        )
    }

    fn current_loop(&self) -> Option<&LoopContext> {
        self.loop_stack.last()
    }

    fn current_loop_mut(&mut self) -> Option<&mut LoopContext> {
        self.loop_stack.last_mut()
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Compiler;
    use crate::ast::{BlockStatement, Expression, Identifier, Statement};
    use crate::bytecode::{lookup_definition, read_operands, Opcode};
    use crate::position::Position;

    fn decode_instructions(compiler: &Compiler) -> Vec<(usize, Opcode, Vec<usize>)> {
        let chunk = compiler.bytecode();
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
    fn compile_block_helper_compiles_ordered_statements() {
        let pos = Position::new(1, 1);
        let block = BlockStatement::new(
            vec![
                Statement::Expression {
                    expression: Expression::IntegerLiteral {
                        value: 1,
                        raw: "1".to_string(),
                        pos,
                    },
                    pos,
                },
                Statement::Return {
                    value: Expression::IntegerLiteral {
                        value: 2,
                        raw: "2".to_string(),
                        pos,
                    },
                    pos,
                },
                Statement::Let {
                    name: Identifier::new("x", pos),
                    value: Expression::IntegerLiteral {
                        value: 3,
                        raw: "3".to_string(),
                        pos,
                    },
                    pos,
                },
            ],
            pos,
        );

        let mut compiler = Compiler::new();
        compiler
            .compile_block(&block)
            .expect("block compilation should succeed");

        let ops = decode_instructions(&compiler)
            .into_iter()
            .map(|(_, op, _)| op)
            .collect::<Vec<_>>();
        assert_eq!(
            ops,
            vec![
                Opcode::Constant,
                Opcode::Pop,
                Opcode::Constant,
                Opcode::ReturnValue,
                Opcode::Constant,
                Opcode::SetGlobal,
            ]
        );
    }
}
