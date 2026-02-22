use crate::compiler::{CompileError, Compiler};
use crate::lexer::Lexer;
use crate::object::ObjectRef;
use crate::parse_error::ParseError;
use crate::parser::Parser;
use crate::runtime_error::RuntimeError;
use crate::token::Token;
use crate::vm::Vm;

#[derive(Debug, Clone)]
pub struct RunOutcome {
    pub result: ObjectRef,
    pub output: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum RunnerError {
    Parse(Vec<ParseError>),
    Compile(CompileError),
    Runtime(RuntimeError),
}

pub fn run_source(source: &str) -> Result<RunOutcome, RunnerError> {
    let mut parser = Parser::new(Lexer::new(source));
    let program = parser.parse_program();
    if !parser.errors().is_empty() {
        return Err(RunnerError::Parse(parser.errors().to_vec()));
    }

    let mut compiler = Compiler::new();
    compiler
        .compile_program(&program)
        .map_err(RunnerError::Compile)?;

    let mut vm = Vm::new(compiler.into_bytecode());
    let result = vm.run().map_err(RunnerError::Runtime)?;
    let output = vm.take_output();
    Ok(RunOutcome { result, output })
}

pub fn tokenize(source: &str) -> Vec<Token> {
    Lexer::new(source).tokenize_all()
}

pub fn format_tokens(source: &str) -> String {
    let tokens = tokenize(source);
    tokens
        .iter()
        .map(|t| format!("{}('{}') @ {}", t.kind, t.literal, t.pos))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn dump_ast(source: &str) -> Result<String, Vec<ParseError>> {
    let mut parser = Parser::new(Lexer::new(source));
    let program = parser.parse_program();
    if !parser.errors().is_empty() {
        return Err(parser.errors().to_vec());
    }
    Ok(program.to_string())
}
