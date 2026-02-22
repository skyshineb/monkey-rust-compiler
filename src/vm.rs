use crate::bytecode::Bytecode;
use crate::object::Object;
use crate::position::Position;
use crate::runtime_error::{RuntimeError, RuntimeErrorKind};

/// Virtual machine placeholder.
#[derive(Debug, Clone)]
pub struct Vm {
    bytecode: Bytecode,
}

impl Vm {
    pub fn new(bytecode: Bytecode) -> Self {
        Self { bytecode }
    }

    pub fn run(&mut self) -> Result<Object, RuntimeError> {
        let _ = &self.bytecode;
        // TODO(step-7): execute bytecode and return final stack value.
        Ok(Object::Null)
    }

    pub fn unsupported_placeholder(&self) -> RuntimeError {
        RuntimeError::new(
            RuntimeErrorKind::UnsupportedOperation,
            "VM operation not implemented yet",
            Position::default(),
        )
    }
}
