#![allow(dead_code)]

pub mod conformance;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use monkey_rust_compiler::parse_error::ParseError;
use monkey_rust_compiler::repl::{format_parse_errors, ReplEvalResult, ReplSession};
use monkey_rust_compiler::runner::{dump_ast, format_tokens, run_source, RunnerError};

pub fn normalize_text(s: &str) -> String {
    let normalized = s.replace("\r\n", "\n");
    let trimmed = normalized.trim_end_matches('\n');
    format!("{trimmed}\n")
}

pub fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("failed reading {}: {e}", path.display()))
}

pub fn assert_or_update_golden(actual: &str, golden_path: &Path) {
    let actual_norm = normalize_text(actual);
    let update = env::var("UPDATE_GOLDENS").ok().as_deref() == Some("1");

    if update {
        if let Some(parent) = golden_path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("failed creating {}: {e}", parent.display()));
        }
        fs::write(golden_path, actual_norm)
            .unwrap_or_else(|e| panic!("failed writing {}: {e}", golden_path.display()));
        return;
    }

    let expected = fs::read_to_string(golden_path).unwrap_or_else(|_| {
        panic!(
            "missing golden file {}. regenerate with UPDATE_GOLDENS=1 cargo test compat_",
            golden_path.display()
        )
    });
    let expected_norm = normalize_text(&expected);

    assert_eq!(
        expected_norm,
        actual_norm,
        "golden mismatch for {}",
        golden_path.display()
    );
}

pub fn fixture_cases(dir: &str, extension: &str) -> Vec<PathBuf> {
    let mut entries = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed reading fixture dir {dir}: {e}"))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some(extension))
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

pub fn golden_for(input: &Path, golden_suffix: &str) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("fixture file must have stem");
    input.with_file_name(format!("{stem}.{golden_suffix}.golden"))
}

pub fn render_tokens(source: &str) -> String {
    format_tokens(source)
}

pub fn render_ast(source: &str) -> String {
    match dump_ast(source) {
        Ok(ast) => format!("STATUS: ok\nAST:\n{ast}"),
        Err(errors) => {
            let lines = errors
                .iter()
                .map(|e| format!("- {e}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("STATUS: parse_error\n{lines}")
        }
    }
}

fn render_parse_errors(errors: &[ParseError]) -> String {
    let lines = errors
        .iter()
        .map(|e| format!("- {e}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{lines}")
}

pub fn render_run(source: &str) -> String {
    match run_source(source) {
        Ok(outcome) => {
            if outcome.output.is_empty() {
                format!(
                    "STATUS: ok\nPUTS: <none>\nRESULT: {}",
                    outcome.result.inspect()
                )
            } else {
                format!(
                    "STATUS: ok\nPUTS:\n{}\nRESULT: {}",
                    outcome.output.join("\n"),
                    outcome.result.inspect()
                )
            }
        }
        Err(RunnerError::Parse(errors)) => format!(
            "STATUS: error\nKIND: parse\nPUTS: <none>\nERROR:\n{}",
            render_parse_errors(&errors)
        ),
        Err(RunnerError::Compile(err)) => {
            format!("STATUS: error\nKIND: compile\nPUTS: <none>\nERROR:\n{err}")
        }
        Err(RunnerError::Runtime(err)) => format!(
            "STATUS: error\nKIND: runtime\nPUTS: <none>\nERROR:\n{}",
            err.format_multiline()
        ),
    }
}

pub fn render_repl_transcript(transcript: &str) -> String {
    let mut repl = ReplSession::new();
    let mut blocks = Vec::new();

    for line in transcript.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let rendered = match repl.eval_line(trimmed) {
            ReplEvalResult::Empty => "(empty)".to_string(),
            ReplEvalResult::Value { result, output } => {
                if output.is_empty() {
                    format!("RESULT: {}", result.inspect())
                } else {
                    format!("PUTS:\n{}\nRESULT: {}", output.join("\n"), result.inspect())
                }
            }
            ReplEvalResult::ParseErrors(errors) => {
                format!("PARSE_ERROR:\n{}", format_parse_errors(&errors))
            }
            ReplEvalResult::CompileError(err) => format!("COMPILE_ERROR:\n{err}"),
            ReplEvalResult::RuntimeError(err) => {
                format!("RUNTIME_ERROR:\n{}", err.format_multiline())
            }
            ReplEvalResult::MetaOutput(text) => format!("META:\n{text}"),
            ReplEvalResult::ExitRequested => "EXIT".to_string(),
        };

        blocks.push(format!("INPUT: {trimmed}\nOUTPUT:\n{rendered}"));
    }

    blocks.join("\n\n")
}
