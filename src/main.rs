use std::env;
use std::fs;
use std::process::ExitCode;
use std::time::Instant;

use monkey_rust_compiler::cli::{parse_args, Command};
use monkey_rust_compiler::repl::ReplSession;
use monkey_rust_compiler::runner::{dump_ast, format_tokens, run_source, RunnerError};

const USAGE: &str = "Usage: monkey [run <path> | bench <path> | --tokens <path> | --ast <path>]";

fn print_usage(stderr: bool) {
    if stderr {
        eprintln!("{USAGE}");
    } else {
        println!("{USAGE}");
    }
}

fn read_file(path: &str) -> Result<String, ExitCode> {
    fs::read_to_string(path).map_err(|err| {
        eprintln!("Failed to read {path}: {err}");
        ExitCode::from(1)
    })
}

fn print_parse_errors(path: &str, errors: &[monkey_rust_compiler::parse_error::ParseError]) {
    eprintln!("Parse errors in {path}:");
    for err in errors {
        eprintln!("- {err}");
    }
}

fn run_file(path: &str, bench: bool) -> ExitCode {
    let source = match read_file(path) {
        Ok(s) => s,
        Err(code) => return code,
    };

    let started = Instant::now();
    match run_source(&source) {
        Ok(outcome) => {
            for line in outcome.output {
                println!("{line}");
            }
            println!("{}", outcome.result.inspect());
            if bench {
                let ms = started.elapsed().as_secs_f64() * 1000.0;
                eprintln!("Execution time: {ms:.2} ms");
            }
            ExitCode::SUCCESS
        }
        Err(RunnerError::Parse(errors)) => {
            print_parse_errors(path, &errors);
            ExitCode::from(1)
        }
        Err(RunnerError::Compile(err)) => {
            eprintln!("Compile error in {path}:");
            eprintln!("{err}");
            ExitCode::from(1)
        }
        Err(RunnerError::Runtime(err)) => {
            eprintln!("Runtime error in {path}:");
            eprintln!("{}", err.format_multiline());
            ExitCode::from(1)
        }
    }
}

fn tokens_file(path: &str) -> ExitCode {
    let source = match read_file(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    println!("{}", format_tokens(&source));
    ExitCode::SUCCESS
}

fn ast_file(path: &str) -> ExitCode {
    let source = match read_file(path) {
        Ok(s) => s,
        Err(code) => return code,
    };

    match dump_ast(&source) {
        Ok(ast) => {
            println!("{ast}");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            print_parse_errors(path, &errors);
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = match parse_args(&args) {
        Ok(cmd) => cmd,
        Err(()) => {
            print_usage(true);
            return ExitCode::from(2);
        }
    };

    match command {
        Command::Help => {
            print_usage(false);
            ExitCode::SUCCESS
        }
        Command::Repl => ExitCode::from(ReplSession::new().run_stdio() as u8),
        Command::Run { path } => run_file(&path, false),
        Command::Bench { path } => run_file(&path, true),
        Command::Tokens { path } => tokens_file(&path),
        Command::Ast { path } => ast_file(&path),
    }
}
