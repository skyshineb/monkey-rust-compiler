use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::ast::{Expression, Program, Statement};
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

    fn unsupported_statement(name: &str, pos: Position) -> Self {
        Self::new(
            format!("unsupported statement in step 10: {name}"),
            Some(pos),
        )
    }

    fn unsupported_expression(name: &str, pos: Position) -> Self {
        Self::new(
            format!("unsupported expression in step 10: {name}"),
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
#[derive(Debug)]
pub struct Compiler {
    chunk: Chunk,
    symbol_table: crate::symbol_table::SymbolTableRef,
}

impl Compiler {
    pub fn new() -> Self {
        let mut root = SymbolTable::new();
        define_builtins(&mut root);

        Self {
            chunk: Chunk::new(),
            symbol_table: root.into_ref(),
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<(), CompileError> {
        for stmt in &program.statements {
            self.compile_statement(stmt)?;
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
            Statement::Return { pos, .. } => {
                return Err(CompileError::unsupported_statement("Return", *pos));
            }
            Statement::While { pos, .. } => {
                return Err(CompileError::unsupported_statement("While", *pos));
            }
            Statement::Break { pos } => {
                return Err(CompileError::unsupported_statement("Break", *pos));
            }
            Statement::Continue { pos } => {
                return Err(CompileError::unsupported_statement("Continue", *pos));
            }
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
                            format!("unsupported prefix operator in step 10: {operator}"),
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
                        // TODO(step-11): implement short-circuit compilation for logical operators.
                        return Err(CompileError::new(
                            "logical operator && compilation is not implemented in step 10",
                            Some(*pos),
                        ));
                    }
                    "||" => {
                        // TODO(step-11): implement short-circuit compilation for logical operators.
                        return Err(CompileError::new(
                            "logical operator || compilation is not implemented in step 10",
                            Some(*pos),
                        ));
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
                            format!("unsupported infix operator in step 10: {operator}"),
                            Some(*pos),
                        ));
                    }
                };
                self.emit(opcode, &[], *pos)?;
            }
            Expression::If { pos, .. } => {
                // TODO(step-13): compile conditional expressions.
                return Err(CompileError::unsupported_expression("If", *pos));
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
        Ok(offset)
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
                        "unsupported function symbol load in step 10: {}",
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
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
