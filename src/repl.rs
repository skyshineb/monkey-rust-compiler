use std::collections::BTreeSet;
use std::io::{self, Write};

use crate::ast::Statement;
use crate::compiler::CompileError;
use crate::lexer::Lexer;
use crate::object::ObjectRef;
use crate::parse_error::ParseError;
use crate::parser::Parser;
use crate::runner::{dump_ast, format_tokens, run_source, RunnerError};
use crate::runtime_error::RuntimeError;

const MONKEY_FACE: &str = "            __,____\n   .--.  .-\"     \"-.  .--.\n  / .. \\/  .-. .-.  \\/ .. \\\n | |  '|  /   Y   \\  |'  | |\n | \\   \\  \\ 0 | 0 /  /   / |\n  \\ '- ,\\.-\"`` ``\"-./, -' /\n   `'-' /_   ^ ^   _\\ '-'`\n       |  \\._   _./  |\n       \\   \\ `~` /   /\n        '._ '-=-' _.'\n           '-----'";

#[derive(Debug, Clone)]
pub enum ReplEvalResult {
    Empty,
    Value {
        result: ObjectRef,
        output: Vec<String>,
    },
    ParseErrors(Vec<ParseError>),
    CompileError(CompileError),
    RuntimeError(RuntimeError),
    MetaOutput(String),
    ExitRequested,
}

/// Stateful REPL session that preserves definitions across inputs.
#[derive(Debug, Default)]
pub struct ReplSession {
    history: Vec<String>,
    bindings: BTreeSet<String>,
    pending_lines: Vec<String>,
}

impl ReplSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eval_line(&mut self, line: &str) -> ReplEvalResult {
        let raw = line.trim_end_matches(['\n', '\r']);
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return ReplEvalResult::Empty;
        }

        if self.pending_lines.is_empty() && trimmed.starts_with(':') {
            return self.eval_meta(trimmed);
        }

        self.pending_lines.push(raw.to_string());
        let pending_source = self.pending_lines.join("\n");
        if !Self::is_complete_source(&pending_source) {
            return ReplEvalResult::Empty;
        }

        let mut all = self.history.clone();
        all.extend(self.pending_lines.iter().cloned());
        let source = all.join("\n");

        let result = match run_source(&source) {
            Ok(outcome) => {
                self.history.extend(self.pending_lines.iter().cloned());
                self.remember_bindings_from_source(&pending_source);
                ReplEvalResult::Value {
                    result: outcome.result,
                    output: outcome.output,
                }
            }
            Err(RunnerError::Parse(errors)) => ReplEvalResult::ParseErrors(errors),
            Err(RunnerError::Compile(err)) => ReplEvalResult::CompileError(err),
            Err(RunnerError::Runtime(err)) => ReplEvalResult::RuntimeError(err),
        };

        self.pending_lines.clear();
        result
    }

    pub fn run_stdio(&mut self) -> i32 {
        let stdin = io::stdin();
        let mut input = String::new();

        loop {
            print!(">> ");
            if io::stdout().flush().is_err() {
                return 1;
            }

            input.clear();
            let read = match stdin.read_line(&mut input) {
                Ok(n) => n,
                Err(_) => return 1,
            };
            if read == 0 {
                return 0;
            }

            match self.eval_line(input.trim_end_matches(['\n', '\r'])) {
                ReplEvalResult::Empty => {}
                ReplEvalResult::Value { result, output } => {
                    for line in output {
                        println!("{line}");
                    }
                    println!("{}", result.inspect());
                }
                ReplEvalResult::ParseErrors(errors) => {
                    println!("{}", format_parse_errors(&errors));
                }
                ReplEvalResult::CompileError(err) => {
                    println!("Compile error:");
                    println!("{err}");
                }
                ReplEvalResult::RuntimeError(err) => {
                    println!("{}", err.format_multiline());
                }
                ReplEvalResult::MetaOutput(text) => {
                    println!("{text}");
                }
                ReplEvalResult::ExitRequested => return 0,
            }
        }
    }

    fn eval_meta(&self, line: &str) -> ReplEvalResult {
        let raw = &line[1..];
        let mut parts = raw.splitn(2, char::is_whitespace);
        let cmd = parts.next().unwrap_or_default();
        let arg = parts.next().unwrap_or("").trim();

        match cmd {
            "help" => ReplEvalResult::MetaOutput(
                "Commands: :help, :tokens [input], :ast [input], :env, :quit, :exit".to_string(),
            ),
            "tokens" => {
                let src = if arg.is_empty() {
                    self.history.last().map(String::as_str).unwrap_or("")
                } else {
                    arg
                };
                if src.is_empty() {
                    ReplEvalResult::MetaOutput("TOKENS:\n  (no input)".to_string())
                } else {
                    let body = format_tokens(src)
                        .lines()
                        .map(|l| format!("  {l}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    ReplEvalResult::MetaOutput(format!("TOKENS:\n{body}"))
                }
            }
            "ast" => {
                let src = if arg.is_empty() {
                    self.history.last().map(String::as_str).unwrap_or("")
                } else {
                    arg
                };
                if src.is_empty() {
                    ReplEvalResult::MetaOutput("AST:\n  (no input)".to_string())
                } else {
                    match dump_ast(src) {
                        Ok(ast) => ReplEvalResult::MetaOutput(format!("AST:\n  {ast}")),
                        Err(errors) => {
                            let body = errors
                                .iter()
                                .map(|e| format!("  - {e}"))
                                .collect::<Vec<_>>()
                                .join("\n");
                            ReplEvalResult::MetaOutput(format!("AST parse errors:\n{body}"))
                        }
                    }
                }
            }
            "env" => ReplEvalResult::MetaOutput(self.render_env()),
            "quit" | "exit" => ReplEvalResult::ExitRequested,
            _ => ReplEvalResult::MetaOutput(format!("Unknown command: :{cmd}")),
        }
    }

    fn remember_bindings_from_source(&mut self, source: &str) {
        let mut parser = Parser::new(Lexer::new(source));
        let program = parser.parse_program();
        if !parser.errors().is_empty() {
            return;
        }

        for stmt in program.statements {
            if let Statement::Let { name, .. } = stmt {
                self.bindings.insert(name.value);
            }
        }
    }

    fn render_env(&self) -> String {
        if self.bindings.is_empty() {
            return "ENV:\n  (empty)".to_string();
        }

        let mut lines = vec!["ENV:".to_string()];
        for name in &self.bindings {
            let value = self.resolve_binding_value(name);
            lines.push(format!("  {name} = {value}"));
        }
        lines.join("\n")
    }

    fn resolve_binding_value(&self, name: &str) -> String {
        let mut all = self.history.clone();
        all.push(format!("{name};"));
        match run_source(&all.join("\n")) {
            Ok(outcome) => outcome.result.inspect(),
            Err(RunnerError::Parse(errs)) => format!("<parse error: {}>", errs.len()),
            Err(RunnerError::Compile(err)) => format!("<compile error: {err}>"),
            Err(RunnerError::Runtime(err)) => {
                format!("<runtime error: {}>", err.error_type.code())
            }
        }
    }

    fn is_complete_source(source: &str) -> bool {
        let mut paren = 0i32;
        let mut brace = 0i32;
        let mut bracket = 0i32;
        let mut in_string = false;

        for line in source.lines() {
            let mut chars = line.chars().peekable();
            while let Some(ch) = chars.next() {
                if in_string {
                    if ch == '"' {
                        in_string = false;
                    }
                    continue;
                }

                if ch == '#' {
                    break;
                }

                match ch {
                    '"' => in_string = true,
                    '(' => paren += 1,
                    ')' => {
                        paren -= 1;
                        if paren < 0 {
                            return true;
                        }
                    }
                    '{' => brace += 1,
                    '}' => {
                        brace -= 1;
                        if brace < 0 {
                            return true;
                        }
                    }
                    '[' => bracket += 1,
                    ']' => {
                        bracket -= 1;
                        if bracket < 0 {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }

        !in_string && paren == 0 && brace == 0 && bracket == 0
    }
}

pub fn format_parse_errors(errors: &[ParseError]) -> String {
    let mut lines = vec![
        MONKEY_FACE.to_string(),
        "Woops! We ran into some monkey business here!".to_string(),
        " parser errors:".to_string(),
    ];
    for err in errors {
        lines.push(format!("  - {err}"));
    }
    lines.join("\n")
}
