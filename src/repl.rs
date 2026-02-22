use std::io::{self, Write};

use crate::compiler::CompileError;
use crate::object::ObjectRef;
use crate::parse_error::ParseError;
use crate::runner::{dump_ast, format_tokens, run_source, RunnerError};
use crate::runtime_error::RuntimeError;

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

/// Stateful REPL session that preserves definitions across lines.
#[derive(Debug, Default)]
pub struct ReplSession {
    history: Vec<String>,
}

impl ReplSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eval_line(&mut self, line: &str) -> ReplEvalResult {
        let line = line.trim();
        if line.is_empty() {
            return ReplEvalResult::Empty;
        }

        if line.starts_with(':') {
            return self.eval_meta(line);
        }

        let mut all = self.history.clone();
        all.push(line.to_string());
        let source = all.join("\n");

        match run_source(&source) {
            Ok(outcome) => {
                self.history.push(line.to_string());
                ReplEvalResult::Value {
                    result: outcome.result,
                    output: outcome.output,
                }
            }
            Err(RunnerError::Parse(errors)) => ReplEvalResult::ParseErrors(errors),
            Err(RunnerError::Compile(err)) => ReplEvalResult::CompileError(err),
            Err(RunnerError::Runtime(err)) => ReplEvalResult::RuntimeError(err),
        }
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
                    println!("Parse errors:");
                    for err in errors {
                        println!("- {err}");
                    }
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
            "env" => {
                // TODO(step-19): surface actual runtime/global environment snapshot in REPL.
                ReplEvalResult::MetaOutput("ENV:\n  (tracked in session source)".to_string())
            }
            "quit" | "exit" => ReplEvalResult::ExitRequested,
            _ => ReplEvalResult::MetaOutput(format!("Unknown command: :{cmd}")),
        }
    }
}
