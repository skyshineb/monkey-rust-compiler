use crate::ast::Program;
use crate::token::Token;

/// Placeholder token rendering for future --tokens mode.
pub fn format_tokens_placeholder(tokens: &[Token]) -> String {
    format!("TOKENS: {} token(s)", tokens.len())
}

/// Placeholder AST rendering for future --ast mode.
pub fn format_ast_placeholder(program: &Program) -> String {
    format!("AST: {program}")
}
