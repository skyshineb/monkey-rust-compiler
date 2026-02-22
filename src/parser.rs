use crate::ast::Program;
use crate::lexer::Lexer;
use crate::parse_error::ParseError;

/// Parser placeholder.
#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
            errors: Vec::new(),
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let _ = self.lexer.input();
        // TODO(step-2): implement Pratt parser and statement parsing.
        Program::default()
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }
}
