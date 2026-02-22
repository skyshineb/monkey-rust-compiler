/// Runtime value placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    String(String),
    Null,
    Placeholder,
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(v) => v.to_string(),
            Object::Boolean(v) => v.to_string(),
            Object::String(v) => v.clone(),
            Object::Null => "null".to_string(),
            Object::Placeholder => "<placeholder>".to_string(),
        }
    }
}
