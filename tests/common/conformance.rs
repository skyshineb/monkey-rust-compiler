use std::env;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConformanceMode {
    Run,
    Tokens,
    Ast,
}

impl ConformanceMode {
    pub fn fixture_dir(&self) -> &'static str {
        match self {
            ConformanceMode::Run => "tests/fixtures/conformance/run",
            ConformanceMode::Tokens => "tests/fixtures/conformance/tokens",
            ConformanceMode::Ast => "tests/fixtures/conformance/ast",
        }
    }

    fn rust_args(&self, path: &Path) -> Vec<String> {
        let p = path.to_string_lossy().to_string();
        match self {
            ConformanceMode::Run => vec!["run".to_string(), p],
            ConformanceMode::Tokens => vec!["--tokens".to_string(), p],
            ConformanceMode::Ast => vec!["--ast".to_string(), p],
        }
    }

    fn java_args(&self, path: &Path) -> Vec<String> {
        self.rust_args(path)
    }

    fn java_capability_env(&self) -> Option<&'static str> {
        match self {
            ConformanceMode::Run => None,
            ConformanceMode::Tokens => Some("MONKEY_JAVA_REF_HAS_TOKENS"),
            ConformanceMode::Ast => Some("MONKEY_JAVA_REF_HAS_AST"),
        }
    }
}

impl Display for ConformanceMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ConformanceMode::Run => write!(f, "run"),
            ConformanceMode::Tokens => write!(f, "tokens"),
            ConformanceMode::Ast => write!(f, "ast"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceMismatch {
    pub fixture: PathBuf,
    pub mode: ConformanceMode,
    pub rust_cmd: String,
    pub java_cmd: String,
    pub rust_out: CommandOutput,
    pub java_out: CommandOutput,
    pub diff: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConformanceOutcome {
    Match,
    Mismatch(ConformanceMismatch),
    Skipped(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandSpec {
    program: String,
    args: Vec<String>,
}

impl CommandSpec {
    fn format_cmdline(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub fn normalize_line_endings(input: &str) -> String {
    input.replace("\r\n", "\n")
}

pub fn trim_line_trailing_space(input: &str) -> String {
    input
        .lines()
        .map(|l| l.trim_end_matches([' ', '\t']))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn normalize_final_newline(input: &str) -> String {
    let trimmed = input.trim_end_matches('\n');
    format!("{trimmed}\n")
}

pub fn normalize_stacktrace_paths(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            if line.contains("/target/") && line.contains(':') {
                line.rsplit('/').next().unwrap_or(line).to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn normalize_output(input: &str) -> String {
    let s = normalize_line_endings(input);
    let s = trim_line_trailing_space(&s);
    let s = normalize_stacktrace_paths(&s);
    normalize_final_newline(&s)
}

pub fn parse_command_line(input: &str) -> Result<(String, Vec<String>), String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;

    for ch in input.chars() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if in_single || in_double {
        return Err("unterminated quoted segment in command".to_string());
    }
    if !current.is_empty() {
        args.push(current);
    }
    if args.is_empty() {
        return Err("empty command".to_string());
    }

    let program = args.remove(0);
    Ok((program, args))
}

fn rust_command_spec(mode: ConformanceMode, fixture: &Path) -> CommandSpec {
    let args = mode.rust_args(fixture);
    if let Ok(bin) = env::var("MONKEY_RUST_BIN") {
        return CommandSpec { program: bin, args };
    }

    let bin = env!("CARGO_BIN_EXE_monkey").to_string();
    CommandSpec { program: bin, args }
}

fn java_command_spec(mode: ConformanceMode, fixture: &Path) -> Result<CommandSpec, String> {
    let Some(cap_env) = mode.java_capability_env() else {
        return java_command_spec_inner(mode, fixture);
    };

    let has_cap = env::var(cap_env).unwrap_or_else(|_| "1".to_string());
    if has_cap == "0" {
        return Err(format!("{cap_env}=0: skipping {mode} parity"));
    }

    java_command_spec_inner(mode, fixture)
}

fn java_command_spec_inner(mode: ConformanceMode, fixture: &Path) -> Result<CommandSpec, String> {
    let cmd = env::var("MONKEY_JAVA_REF_CMD")
        .map_err(|_| "MONKEY_JAVA_REF_CMD is not set".to_string())?;
    let (program, mut args) = parse_command_line(&cmd)?;
    args.extend(mode.java_args(fixture));
    Ok(CommandSpec { program, args })
}

fn run_command(spec: &CommandSpec) -> Result<CommandOutput, String> {
    let output = Command::new(&spec.program)
        .args(&spec.args)
        .output()
        .map_err(|e| format!("failed to run '{}': {e}", spec.format_cmdline()))?;

    Ok(CommandOutput {
        stdout: normalize_output(&String::from_utf8_lossy(&output.stdout)),
        stderr: normalize_output(&String::from_utf8_lossy(&output.stderr)),
        status: output.status.code().unwrap_or(-1),
    })
}

pub fn unified_diff(left_label: &str, left: &str, right_label: &str, right: &str) -> String {
    let left_lines = left.lines().collect::<Vec<_>>();
    let right_lines = right.lines().collect::<Vec<_>>();
    let max_len = left_lines.len().max(right_lines.len());

    let mut out = vec![format!("--- {left_label}"), format!("+++ {right_label}")];
    for i in 0..max_len {
        match (left_lines.get(i), right_lines.get(i)) {
            (Some(l), Some(r)) if l == r => out.push(format!(" {l}")),
            (Some(l), Some(r)) => {
                out.push(format!("-{l}"));
                out.push(format!("+{r}"));
            }
            (Some(l), None) => out.push(format!("-{l}")),
            (None, Some(r)) => out.push(format!("+{r}")),
            (None, None) => {}
        }
    }
    out.join("\n")
}

pub fn fixture_cases(dir: &str) -> Vec<PathBuf> {
    let mut entries = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed reading fixture dir {dir}: {e}"))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("monkey"))
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

pub fn compare_fixture(mode: ConformanceMode, fixture: &Path) -> ConformanceOutcome {
    let rust_spec = rust_command_spec(mode, fixture);
    let java_spec = match java_command_spec(mode, fixture) {
        Ok(spec) => spec,
        Err(reason) => return ConformanceOutcome::Skipped(reason),
    };

    let rust_out = match run_command(&rust_spec) {
        Ok(o) => o,
        Err(e) => return ConformanceOutcome::Skipped(e),
    };
    let java_out = match run_command(&java_spec) {
        Ok(o) => o,
        Err(e) => return ConformanceOutcome::Skipped(e),
    };

    let rust_success = rust_out.status == 0;
    let java_success = java_out.status == 0;

    let stdout_match = rust_out.stdout == java_out.stdout;
    let stderr_match = rust_out.stderr == java_out.stderr;
    let status_shape_match = rust_success == java_success;

    if stdout_match && stderr_match && status_shape_match {
        return ConformanceOutcome::Match;
    }

    let mut diff_parts = Vec::new();
    if !stdout_match {
        diff_parts.push(unified_diff(
            "rust.stdout",
            &rust_out.stdout,
            "java.stdout",
            &java_out.stdout,
        ));
    }
    if !stderr_match {
        diff_parts.push(unified_diff(
            "rust.stderr",
            &rust_out.stderr,
            "java.stderr",
            &java_out.stderr,
        ));
    }
    if !status_shape_match {
        diff_parts.push(format!(
            "status mismatch: rust={} java={} (success-shape)",
            rust_out.status, java_out.status
        ));
    }

    ConformanceOutcome::Mismatch(ConformanceMismatch {
        fixture: fixture.to_path_buf(),
        mode,
        rust_cmd: rust_spec.format_cmdline(),
        java_cmd: java_spec.format_cmdline(),
        rust_out,
        java_out,
        diff: diff_parts.join("\n\n"),
    })
}

pub fn compare_rust_to_rust(mode: ConformanceMode, fixture: &Path) -> Result<String, String> {
    let spec = rust_command_spec(mode, fixture);
    let left = run_command(&spec)?;
    let right = run_command(&spec)?;

    if left.stdout == right.stdout && left.stderr == right.stderr {
        Ok("match".to_string())
    } else {
        Err(unified_diff(
            "rust.left",
            &left.stdout,
            "rust.right",
            &right.stdout,
        ))
    }
}
