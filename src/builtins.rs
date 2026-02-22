use crate::object::{Object, ObjectRef};
use crate::runtime_error::RuntimeErrorType;

/// Stable builtin names expected by compatibility contract.
pub fn builtin_names() -> &'static [&'static str] {
    &["len", "first", "last", "rest", "push", "puts"]
}

pub fn builtin_name_at(index: usize) -> Option<&'static str> {
    builtin_names().get(index).copied()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinError {
    pub error_type: RuntimeErrorType,
    pub message: String,
}

impl BuiltinError {
    fn wrong_arg_count(name: &str, expected: usize, got: usize) -> Self {
        Self {
            error_type: RuntimeErrorType::WrongArgumentCount,
            message: format!("{name} expected {expected} argument(s), got {got}"),
        }
    }

    fn invalid_arg_type(name: &str, expected: &str, got: &str) -> Self {
        Self {
            error_type: RuntimeErrorType::InvalidArgumentType,
            message: format!("{name} expected {expected}, got {got}"),
        }
    }
}

pub fn execute_builtin(
    name: &str,
    args: &[ObjectRef],
    output: &mut Vec<String>,
) -> Result<ObjectRef, BuiltinError> {
    match name {
        "len" => {
            if args.len() != 1 {
                return Err(BuiltinError::wrong_arg_count("len", 1, args.len()));
            }
            match args[0].as_ref() {
                Object::String(v) => Ok(Object::Integer(v.chars().count() as i64).rc()),
                Object::Array(values) => Ok(Object::Integer(values.len() as i64).rc()),
                other => Err(BuiltinError::invalid_arg_type(
                    "len",
                    "STRING or ARRAY",
                    other.type_name(),
                )),
            }
        }
        "first" => {
            if args.len() != 1 {
                return Err(BuiltinError::wrong_arg_count("first", 1, args.len()));
            }
            match args[0].as_ref() {
                Object::Array(values) => {
                    Ok(values.first().cloned().unwrap_or_else(|| Object::Null.rc()))
                }
                other => Err(BuiltinError::invalid_arg_type(
                    "first",
                    "ARRAY",
                    other.type_name(),
                )),
            }
        }
        "last" => {
            if args.len() != 1 {
                return Err(BuiltinError::wrong_arg_count("last", 1, args.len()));
            }
            match args[0].as_ref() {
                Object::Array(values) => {
                    Ok(values.last().cloned().unwrap_or_else(|| Object::Null.rc()))
                }
                other => Err(BuiltinError::invalid_arg_type(
                    "last",
                    "ARRAY",
                    other.type_name(),
                )),
            }
        }
        "rest" => {
            if args.len() != 1 {
                return Err(BuiltinError::wrong_arg_count("rest", 1, args.len()));
            }
            match args[0].as_ref() {
                Object::Array(values) => {
                    if values.is_empty() {
                        Ok(Object::Null.rc())
                    } else {
                        Ok(Object::Array(values[1..].to_vec()).rc())
                    }
                }
                other => Err(BuiltinError::invalid_arg_type(
                    "rest",
                    "ARRAY",
                    other.type_name(),
                )),
            }
        }
        "push" => {
            if args.len() != 2 {
                return Err(BuiltinError::wrong_arg_count("push", 2, args.len()));
            }
            match args[0].as_ref() {
                Object::Array(values) => {
                    let mut out = values.clone();
                    out.push(args[1].clone());
                    Ok(Object::Array(out).rc())
                }
                other => Err(BuiltinError::invalid_arg_type(
                    "push",
                    "ARRAY",
                    other.type_name(),
                )),
            }
        }
        "puts" => {
            let line = args
                .iter()
                .map(|arg| arg.inspect())
                .collect::<Vec<_>>()
                .join("");
            output.push(line);
            Ok(Object::Null.rc())
        }
        _ => Err(BuiltinError {
            error_type: RuntimeErrorType::UnsupportedOperation,
            message: format!("unknown builtin: {name}"),
        }),
    }
}
