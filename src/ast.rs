use crate::position::Position;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Program root node.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub value: String,
    pub pos: Position,
}

impl Identifier {
    pub fn new(value: impl Into<String>, pos: Position) -> Self {
        Self {
            value: value.into(),
            pos,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
    pub pos: Position,
}

impl BlockStatement {
    pub fn new(statements: Vec<Statement>, pos: Position) -> Self {
        Self { statements, pos }
    }

    pub fn pos(&self) -> Position {
        self.pos
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let {
        name: Identifier,
        value: Expression,
        pos: Position,
    },
    Return {
        value: Expression,
        pos: Position,
    },
    While {
        condition: Expression,
        body: BlockStatement,
        pos: Position,
    },
    Break {
        pos: Position,
    },
    Continue {
        pos: Position,
    },
    Expression {
        expression: Expression,
        pos: Position,
    },
}

impl Statement {
    pub fn pos(&self) -> Position {
        match self {
            Statement::Let { pos, .. }
            | Statement::Return { pos, .. }
            | Statement::While { pos, .. }
            | Statement::Break { pos }
            | Statement::Continue { pos }
            | Statement::Expression { pos, .. } => *pos,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Identifier {
        value: String,
        pos: Position,
    },
    IntegerLiteral {
        value: i64,
        raw: String,
        pos: Position,
    },
    BooleanLiteral {
        value: bool,
        pos: Position,
    },
    StringLiteral {
        value: String,
        pos: Position,
    },
    Prefix {
        operator: String,
        right: Box<Expression>,
        pos: Position,
    },
    Infix {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
        pos: Position,
    },
    If {
        condition: Box<Expression>,
        consequence: BlockStatement,
        alternative: Option<BlockStatement>,
        pos: Position,
    },
    FunctionLiteral {
        parameters: Vec<Identifier>,
        body: BlockStatement,
        pos: Position,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
        pos: Position,
    },
    ArrayLiteral {
        elements: Vec<Expression>,
        pos: Position,
    },
    HashLiteral {
        pairs: Vec<(Expression, Expression)>,
        pos: Position,
    },
    Index {
        left: Box<Expression>,
        index: Box<Expression>,
        pos: Position,
    },
}

impl Expression {
    pub fn pos(&self) -> Position {
        match self {
            Expression::Identifier { pos, .. }
            | Expression::IntegerLiteral { pos, .. }
            | Expression::BooleanLiteral { pos, .. }
            | Expression::StringLiteral { pos, .. }
            | Expression::Prefix { pos, .. }
            | Expression::Infix { pos, .. }
            | Expression::If { pos, .. }
            | Expression::FunctionLiteral { pos, .. }
            | Expression::Call { pos, .. }
            | Expression::ArrayLiteral { pos, .. }
            | Expression::HashLiteral { pos, .. }
            | Expression::Index { pos, .. } => *pos,
        }
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for (idx, stmt) in self.statements.iter().enumerate() {
            if idx > 0 {
                writeln!(f)?;
            }
            write!(f, "{stmt}")?;
        }
        Ok(())
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

impl Display for BlockStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.statements.is_empty() {
            return write!(f, "{{}}");
        }

        write!(f, "{{ ")?;
        for (idx, stmt) in self.statements.iter().enumerate() {
            if idx > 0 {
                write!(f, " ")?;
            }
            write!(f, "{stmt}")?;
        }
        write!(f, " }}")
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Statement::Let { name, value, .. } => write!(f, "let {name} = {value};"),
            Statement::Return { value, .. } => write!(f, "return {value};"),
            Statement::While {
                condition, body, ..
            } => write!(f, "while ({condition}) {body}"),
            Statement::Break { .. } => write!(f, "break;"),
            Statement::Continue { .. } => write!(f, "continue;"),
            Statement::Expression { expression, .. } => write!(f, "{expression};"),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // TODO(step-5): parser should construct these nodes in source order and precedence shape.
        match self {
            Expression::Identifier { value, .. } => write!(f, "{value}"),
            Expression::IntegerLiteral { raw, .. } => write!(f, "{raw}"),
            Expression::BooleanLiteral { value, .. } => write!(f, "{value}"),
            Expression::StringLiteral { value, .. } => write!(f, "\"{value}\""),
            Expression::Prefix {
                operator, right, ..
            } => write!(f, "({operator}{right})"),
            Expression::Infix {
                left,
                operator,
                right,
                ..
            } => write!(f, "({left} {operator} {right})"),
            Expression::If {
                condition,
                consequence,
                alternative,
                ..
            } => match alternative {
                Some(alt) => write!(f, "if ({condition}) {consequence} else {alt}"),
                None => write!(f, "if ({condition}) {consequence}"),
            },
            Expression::FunctionLiteral {
                parameters, body, ..
            } => {
                let params = parameters
                    .iter()
                    .map(|p| p.value.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "fn({params}) {body}")
            }
            Expression::Call {
                function,
                arguments,
                ..
            } => {
                let args = arguments
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{function}({args})")
            }
            Expression::ArrayLiteral { elements, .. } => {
                let rendered = elements
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "[{rendered}]")
            }
            Expression::HashLiteral { pairs, .. } => {
                let rendered = pairs
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{{rendered}}}")
            }
            Expression::Index { left, index, .. } => write!(f, "({left}[{index}])"),
        }
    }
}
