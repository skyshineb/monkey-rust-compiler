use crate::ast::Program;
use crate::bytecode::Bytecode;

/// Compiler placeholder.
#[derive(Debug, Default)]
pub struct Compiler {
    bytecode: Bytecode,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::empty(),
        }
    }

    pub fn compile(&mut self, _program: &Program) -> Result<(), String> {
        // TODO(step-6): compile AST into bytecode instructions.
        Ok(())
    }

    pub fn bytecode(&self) -> Bytecode {
        self.bytecode.clone()
    }
}
