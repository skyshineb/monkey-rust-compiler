use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;

use crate::ast::{BlockStatement, Expression, Identifier, Program, Statement};
use crate::bytecode::{make, BytecodeError, Chunk, Opcode};
use crate::object::{CompiledFunctionObject, Object};
use crate::position::Position;
use crate::symbol_table::{define_builtins, Symbol, SymbolScope, SymbolTable, SymbolTableRef};

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
            format!("unsupported expression in step 14: {name}"),
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

#[derive(Debug, Clone, Default)]
struct CompilationScope {
    instructions: Vec<u8>,
    positions: Vec<(usize, Position)>,
    last_instruction: Option<EmittedInstruction>,
    previous_instruction: Option<EmittedInstruction>,
    loop_stack: Vec<LoopContext>,
}

/// Compiler for Monkey bytecode.
#[derive(Debug)]
pub struct Compiler {
    chunk: Chunk,
    symbol_table: SymbolTableRef,
    last_instruction: Option<EmittedInstruction>,
    previous_instruction: Option<EmittedInstruction>,
    loop_stack: Vec<LoopContext>,
    scopes: Vec<CompilationScope>,
    scope_index: usize,
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
            scopes: Vec::new(),
            scope_index: 0,
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
                match value {
                    Expression::FunctionLiteral {
                        parameters,
                        body,
                        pos: fn_pos,
                    } => self.compile_function_literal(
                        parameters,
                        body,
                        *fn_pos,
                        Some(name.value.clone()),
                    )?,
                    _ => self.compile_expression(value)?,
                }

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
                self.current_loop_stack_mut().push(LoopContext {
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

                let loop_ctx = self.current_loop_stack_mut().pop().ok_or_else(|| {
                    CompileError::new("while loop context stack underflow", Some(*pos))
                })?;
                for break_jump in loop_ctx.break_jumps {
                    self.patch_jump(break_jump, loop_end)?;
                }
            }
            Statement::Break { pos } => {
                if self.current_loop_stack().is_empty() {
                    // TODO(step-17): VM will translate this opcode into INVALID_CONTROL_FLOW.
                    self.emit(Opcode::InvalidBreak, &[], *pos)?;
                } else {
                    let break_jump = self.emit_jump(Opcode::Jump, *pos)?;
                    if let Some(loop_ctx) = self.current_loop_stack_mut().last_mut() {
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
                if let Some(loop_ctx) = self.current_loop_stack().last() {
                    self.emit(Opcode::Jump, &[loop_ctx.continue_target], *pos)?;
                } else {
                    // TODO(step-17): VM will translate this opcode into INVALID_CONTROL_FLOW.
                    self.emit(Opcode::InvalidContinue, &[], *pos)?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn compile_block(&mut self, block: &BlockStatement) -> Result<(), CompileError> {
        // TODO(step-14): function-body compilation reuses statement-context block compilation.
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
                            format!("unsupported prefix operator in step 14: {operator}"),
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
                            format!("unsupported infix operator in step 14: {operator}"),
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
            Expression::FunctionLiteral {
                parameters,
                body,
                pos,
            } => {
                self.compile_function_literal(parameters, body, *pos, None)?;
            }
            Expression::Call {
                function,
                arguments,
                pos,
            } => {
                self.compile_expression(function)?;
                for arg in arguments {
                    self.compile_expression(arg)?;
                }
                self.emit(Opcode::Call, &[arguments.len()], *pos)?;
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

    fn compile_function_literal(
        &mut self,
        parameters: &[Identifier],
        body: &BlockStatement,
        pos: Position,
        inferred_name: Option<String>,
    ) -> Result<(), CompileError> {
        self.enter_scope();

        if let Some(name) = &inferred_name {
            self.symbol_table
                .borrow_mut()
                .define_function_name(name.clone());
        }

        for param in parameters {
            self.symbol_table.borrow_mut().define(param.value.clone());
        }

        self.compile_block(body)?;

        if self.last_instruction_is(Opcode::Pop) {
            self.replace_last_pop_with_return_value(pos)?;
        } else if !self.last_instruction_is(Opcode::ReturnValue)
            && !self.last_instruction_is(Opcode::Return)
        {
            self.emit(Opcode::Return, &[], pos)?;
        }

        let free_symbols = self.symbol_table.borrow().free_symbols.clone();
        let num_locals = self.symbol_table.borrow().num_definitions;
        let num_params = parameters.len();

        let scope = self.leave_scope()?;

        for free in &free_symbols {
            self.emit_for_symbol_load(free, pos)?;
        }

        let function = Object::CompiledFunction(Rc::new(CompiledFunctionObject {
            name: inferred_name,
            num_params,
            num_locals,
            instructions: scope.instructions,
            positions: scope.positions,
        }));

        let const_idx = self.add_constant(function, pos);
        self.emit(Opcode::Closure, &[const_idx, free_symbols.len()], pos)?;
        Ok(())
    }

    fn enter_scope(&mut self) {
        self.scopes.push(CompilationScope::default());
        self.scope_index += 1;

        let enclosed = SymbolTable::new_enclosed(self.symbol_table.clone()).into_ref();
        self.symbol_table = enclosed;
    }

    fn leave_scope(&mut self) -> Result<CompilationScope, CompileError> {
        if self.scope_index == 0 {
            return Err(CompileError::new(
                "cannot leave compiler scope: already at root scope",
                None,
            ));
        }

        let scope = self.scopes.pop().ok_or_else(|| {
            CompileError::new("cannot leave compiler scope: scope stack underflow", None)
        })?;
        self.scope_index -= 1;

        let outer = self.symbol_table.borrow().outer.clone().ok_or_else(|| {
            CompileError::new("cannot leave scope: missing outer symbol table", None)
        })?;
        self.symbol_table = outer;

        Ok(scope)
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
        let offset = self.current_offset();
        self.current_instructions_mut().extend_from_slice(&bytes);
        self.current_positions_mut().push((offset, pos));
        self.current_positions_mut().sort_by_key(|(off, _)| *off);
        self.set_last_instruction(op, offset);
        Ok(offset)
    }

    fn current_offset(&self) -> usize {
        self.current_instructions().len()
    }

    fn emit_jump(&mut self, op: Opcode, pos: Position) -> Result<usize, CompileError> {
        self.emit(op, &[0], pos)
    }

    fn patch_jump(&mut self, jump_offset: usize, target_offset: usize) -> Result<(), CompileError> {
        if jump_offset >= self.current_instructions().len() {
            return Err(CompileError::new(
                format!(
                    "invalid jump patch offset {} for instruction length {}",
                    jump_offset,
                    self.current_instructions().len()
                ),
                None,
            ));
        }

        let opcode_byte = self.current_instructions()[jump_offset];
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
        if end > self.current_instructions().len() {
            return Err(CompileError::new(
                format!(
                    "patched jump overflows instruction buffer: {}..{} of {}",
                    jump_offset,
                    end,
                    self.current_instructions().len()
                ),
                None,
            ));
        }

        self.current_instructions_mut()[jump_offset..end].copy_from_slice(&patched);
        Ok(())
    }

    fn replace_instruction(&mut self, offset: usize, bytes: &[u8]) -> Result<(), CompileError> {
        let end = offset + bytes.len();
        if end > self.current_instructions().len() {
            return Err(CompileError::new(
                format!(
                    "replacement instruction out of bounds: {}..{} of {}",
                    offset,
                    end,
                    self.current_instructions().len()
                ),
                None,
            ));
        }
        self.current_instructions_mut()[offset..end].copy_from_slice(bytes);
        Ok(())
    }

    fn remove_last_instruction(&mut self) -> Result<(), CompileError> {
        let Some(last) = self.current_last_instruction() else {
            return Err(CompileError::new(
                "cannot remove last instruction: no instructions emitted",
                None,
            ));
        };

        self.current_instructions_mut().truncate(last.offset);
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
        if self.scope_index == 0 {
            self.previous_instruction = self.last_instruction;
            self.last_instruction = Some(EmittedInstruction { opcode, offset });
        } else if let Some(scope) = self.scopes.last_mut() {
            scope.previous_instruction = scope.last_instruction;
            scope.last_instruction = Some(EmittedInstruction { opcode, offset });
        }
    }

    fn record_last_instruction_from_tail(&mut self) -> Result<(), CompileError> {
        let mut decoded = Vec::new();
        let mut offset = 0;
        let instructions = self.current_instructions().to_vec();

        while offset < instructions.len() {
            let byte = instructions[offset];
            let Some(opcode) = Opcode::from_byte(byte) else {
                return Err(CompileError::new(
                    format!("unknown opcode byte {byte} at offset {offset}"),
                    None,
                ));
            };
            let def = crate::bytecode::lookup_definition(opcode);
            let operand_len: usize = def.operand_widths.iter().sum();
            let end = offset + 1 + operand_len;
            if end > instructions.len() {
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

        let last = decoded.last().copied();
        let prev = if decoded.len() > 1 {
            Some(decoded[decoded.len() - 2])
        } else {
            None
        };

        if self.scope_index == 0 {
            self.last_instruction = last;
            self.previous_instruction = prev;
        } else if let Some(scope) = self.scopes.last_mut() {
            scope.last_instruction = last;
            scope.previous_instruction = prev;
        }
        Ok(())
    }

    fn current_last_instruction(&self) -> Option<EmittedInstruction> {
        if self.scope_index == 0 {
            self.last_instruction
        } else {
            self.scopes.last().and_then(|s| s.last_instruction)
        }
    }

    fn last_instruction_is(&self, opcode: Opcode) -> bool {
        self.current_last_instruction()
            .map(|ins| ins.opcode == opcode)
            .unwrap_or(false)
    }

    fn replace_last_pop_with_return_value(&mut self, pos: Position) -> Result<(), CompileError> {
        let Some(last) = self.current_last_instruction() else {
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
        self.set_last_instruction(Opcode::ReturnValue, last.offset);
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
                self.emit(Opcode::CurrentClosure, &[], pos)?;
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

    fn current_instructions(&self) -> &Vec<u8> {
        if self.scope_index == 0 {
            &self.chunk.instructions
        } else {
            &self.scopes[self.scope_index - 1].instructions
        }
    }

    fn current_instructions_mut(&mut self) -> &mut Vec<u8> {
        if self.scope_index == 0 {
            &mut self.chunk.instructions
        } else {
            &mut self.scopes[self.scope_index - 1].instructions
        }
    }

    fn current_positions_mut(&mut self) -> &mut Vec<(usize, Position)> {
        if self.scope_index == 0 {
            &mut self.chunk.positions
        } else {
            &mut self.scopes[self.scope_index - 1].positions
        }
    }

    fn current_loop_stack(&self) -> &Vec<LoopContext> {
        if self.scope_index == 0 {
            &self.loop_stack
        } else {
            &self.scopes[self.scope_index - 1].loop_stack
        }
    }

    fn current_loop_stack_mut(&mut self) -> &mut Vec<LoopContext> {
        if self.scope_index == 0 {
            &mut self.loop_stack
        } else {
            &mut self.scopes[self.scope_index - 1].loop_stack
        }
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
