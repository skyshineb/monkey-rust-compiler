use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::position::Position;

/// Protocol-compatible runtime error categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeErrorType {
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

impl RuntimeErrorType {
    pub fn code(&self) -> &'static str {
        match self {
            RuntimeErrorType::TypeMismatch => "TYPE_MISMATCH",
            RuntimeErrorType::UnknownIdentifier => "UNKNOWN_IDENTIFIER",
            RuntimeErrorType::NotCallable => "NOT_CALLABLE",
            RuntimeErrorType::WrongArgumentCount => "WRONG_ARGUMENT_COUNT",
            RuntimeErrorType::InvalidArgumentType => "INVALID_ARGUMENT_TYPE",
            RuntimeErrorType::InvalidControlFlow => "INVALID_CONTROL_FLOW",
            RuntimeErrorType::InvalidIndex => "INVALID_INDEX",
            RuntimeErrorType::Unhashable => "UNHASHABLE",
            RuntimeErrorType::DivisionByZero => "DIVISION_BY_ZERO",
            RuntimeErrorType::UnsupportedOperation => "UNSUPPORTED_OPERATION",
        }
    }
}

impl Display for RuntimeErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.code())
    }
}

/// Runtime stack frame information for rich error reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrameInfo {
    pub function_name: String,
    pub pos: Position,
    pub arg_count: Option<usize>,
}

impl StackFrameInfo {
    pub fn new(function_name: impl Into<String>, pos: Position) -> Self {
        Self {
            function_name: function_name.into(),
            pos,
            arg_count: None,
        }
    }

    pub fn with_arg_count(mut self, arg_count: usize) -> Self {
        self.arg_count = Some(arg_count);
        self
    }

    pub fn format_frame(&self) -> String {
        match self.arg_count {
            Some(n) => format!("at {}({} args) @ {}", self.function_name, n, self.pos),
            None => format!("at {} @ {}", self.function_name, self.pos),
        }
    }
}

/// Structured runtime error with source position and optional stack trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub error_type: RuntimeErrorType,
    pub message: String,
    pub pos: Position,
    pub stack: Vec<StackFrameInfo>,
}

impl RuntimeError {
    pub fn new(error_type: RuntimeErrorType, message: impl Into<String>, pos: Position) -> Self {
        Self {
            error_type,
            message: message.into(),
            pos,
            stack: Vec::new(),
        }
    }

    pub fn at(error_type: RuntimeErrorType, pos: Position, message: impl Into<String>) -> Self {
        Self::new(error_type, message, pos)
    }

    pub fn with_stack(mut self, stack: Vec<StackFrameInfo>) -> Self {
        self.stack = stack;
        self
    }

    pub fn with_frame(mut self, frame: StackFrameInfo) -> Self {
        self.stack.push(frame);
        self
    }

    pub fn push_frame(&mut self, frame: StackFrameInfo) {
        self.stack.push(frame);
    }

    pub fn format_single_line(&self) -> String {
        format!(
            "Error[{}] at {}: {}",
            self.error_type, self.pos, self.message
        )
    }

    pub fn format_multiline(&self) -> String {
        if self.stack.is_empty() {
            return self.format_single_line();
        }

        // TODO(step-17): VM will attach instruction-level call frames and source positions.
        let frames = self
            .stack
            .iter()
            .map(|frame| format!("  {}", frame.format_frame()))
            .collect::<Vec<_>>()
            .join("\n");

        format!("{}\nStack trace:\n{}", self.format_single_line(), frames)
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.format_single_line())
    }
}
