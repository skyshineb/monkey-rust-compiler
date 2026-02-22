use std::fmt::{Display, Formatter, Result as FmtResult};

/// Program root node.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Expression(Expression),
    Let { name: String },
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Identifier(String),
    Integer(i64),
    Placeholder,
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Program(statements={})", self.statements.len())
    }
}
