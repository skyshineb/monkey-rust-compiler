use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::position::Position;

/// Runtime error kinds aligned with compatibility contract names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    TypeMismatch,
    UnknownIdentifier,
    NotCallable,
    WrongArgumentCount,
    InvalidArgumentType,
    InvalidControlFlow,
    InvalidIndex,
    Unhashable,
    DivisionByZero,
    UnsupportedOperation,
}

impl RuntimeErrorKind {
    pub fn as_contract_name(&self) -> &'static str {
        match self {
            RuntimeErrorKind::TypeMismatch => "TYPE_MISMATCH",
            RuntimeErrorKind::UnknownIdentifier => "UNKNOWN_IDENTIFIER",
            RuntimeErrorKind::NotCallable => "NOT_CALLABLE",
            RuntimeErrorKind::WrongArgumentCount => "WRONG_ARGUMENT_COUNT",
            RuntimeErrorKind::InvalidArgumentType => "INVALID_ARGUMENT_TYPE",
            RuntimeErrorKind::InvalidControlFlow => "INVALID_CONTROL_FLOW",
            RuntimeErrorKind::InvalidIndex => "INVALID_INDEX",
            RuntimeErrorKind::Unhashable => "UNHASHABLE",
            RuntimeErrorKind::DivisionByZero => "DIVISION_BY_ZERO",
            RuntimeErrorKind::UnsupportedOperation => "UNSUPPORTED_OPERATION",
        }
    }
}

/// Runtime error placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub message: String,
    pub position: Position,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind, message: impl Into<String>, position: Position) -> Self {
        Self {
            kind,
            message: message.into(),
            position,
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // TODO(step-4): add multiline stack trace formatting.
        write!(
            f,
            "Error[{}] at {}: {}",
            self.kind.as_contract_name(),
            self.position,
            self.message
        )
    }
}
