use crate::object::Object;

/// Raw VM instruction bytes.
pub type Instructions = Vec<u8>;

/// Bytecode output placeholder for compiler step.
#[derive(Debug, Clone, Default)]
pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

impl Bytecode {
    pub fn empty() -> Self {
        Self::default()
    }
}
