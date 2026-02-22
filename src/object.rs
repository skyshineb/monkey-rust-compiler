use crate::position::Position;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;

/// Shared reference type for runtime objects.
pub type ObjectRef = Rc<Object>;

/// Hashable Monkey runtime key types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashKey {
    Integer(i64),
    Boolean(bool),
    String(String),
}

/// Placeholder compiled function metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledFunctionObject {
    pub name: Option<String>,
    pub num_params: usize,
    pub num_locals: usize,
    pub instructions: Vec<u8>,
    pub positions: Vec<(usize, Position)>,
}

/// Placeholder closure object metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureObject {
    pub function: Rc<CompiledFunctionObject>,
    pub free: Vec<ObjectRef>,
}

/// Placeholder builtin object metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinObject {
    pub name: String,
}

/// Runtime object model used by the VM.
#[derive(Debug, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    String(String),
    Null,
    Array(Vec<ObjectRef>),
    Hash(Vec<(ObjectRef, ObjectRef)>),
    CompiledFunction(Rc<CompiledFunctionObject>),
    Closure(Rc<ClosureObject>),
    Builtin(BuiltinObject),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Integer(a), Object::Integer(b)) => a == b,
            (Object::Boolean(a), Object::Boolean(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            (Object::Null, Object::Null) => true,
            (Object::Array(a), Object::Array(b)) => a == b,
            (Object::Hash(a), Object::Hash(b)) => a == b,
            (Object::CompiledFunction(a), Object::CompiledFunction(b)) => a == b,
            (Object::Closure(a), Object::Closure(b)) => a == b,
            (Object::Builtin(a), Object::Builtin(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Object {}

impl Object {
    pub fn rc(self) -> ObjectRef {
        Rc::new(self)
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Object::Integer(_) => "INTEGER",
            Object::Boolean(_) => "BOOLEAN",
            Object::String(_) => "STRING",
            Object::Null => "NULL",
            Object::Array(_) => "ARRAY",
            Object::Hash(_) => "HASH",
            Object::CompiledFunction(_) => "FUNCTION",
            Object::Closure(_) => "CLOSURE",
            Object::Builtin(_) => "BUILTIN",
        }
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, Object::Boolean(false) | Object::Null)
    }

    pub fn hash_key(&self) -> Option<HashKey> {
        match self {
            Object::Integer(v) => Some(HashKey::Integer(*v)),
            Object::Boolean(v) => Some(HashKey::Boolean(*v)),
            Object::String(v) => Some(HashKey::String(v.clone())),
            _ => None,
        }
    }

    pub fn inspect(&self) -> String {
        // TODO(step-7): runtime error wiring (e.g., UNHASHABLE/type checks) will use this model.
        match self {
            Object::Integer(v) => v.to_string(),
            Object::Boolean(v) => v.to_string(),
            Object::String(v) => v.clone(),
            Object::Null => "null".to_string(),
            Object::Array(values) => {
                let rendered = values
                    .iter()
                    .map(|v| v.inspect())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{rendered}]")
            }
            Object::Hash(pairs) => {
                let rendered = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.inspect(), v.inspect()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{rendered}}}")
            }
            Object::CompiledFunction(function) => match &function.name {
                Some(name) => format!("<compiled fn:{name}>"),
                None => "<compiled fn>".to_string(),
            },
            Object::Closure(_) => "<closure>".to_string(),
            Object::Builtin(builtin) => format!("<builtin: {}>", builtin.name),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.inspect())
    }
}
