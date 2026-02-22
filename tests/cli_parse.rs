use monkey_rust_compiler::cli::{parse_args, Command};

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

#[test]
fn parses_valid_commands() {
    assert_eq!(parse_args(&args(&[])), Ok(Command::Repl));
    assert_eq!(parse_args(&args(&["repl"])), Ok(Command::Repl));
    assert_eq!(
        parse_args(&args(&["run", "a.monkey"])),
        Ok(Command::Run {
            path: "a.monkey".to_string()
        })
    );
    assert_eq!(
        parse_args(&args(&["bench", "a.monkey"])),
        Ok(Command::Bench {
            path: "a.monkey".to_string()
        })
    );
    assert_eq!(
        parse_args(&args(&["--tokens", "a.monkey"])),
        Ok(Command::Tokens {
            path: "a.monkey".to_string()
        })
    );
    assert_eq!(
        parse_args(&args(&["--ast", "a.monkey"])),
        Ok(Command::Ast {
            path: "a.monkey".to_string()
        })
    );
}

#[test]
fn invalid_combinations_return_usage_error() {
    assert!(parse_args(&args(&["run"])).is_err());
    assert!(parse_args(&args(&["--tokens"])).is_err());
    assert!(parse_args(&args(&["unknown"])).is_err());
    assert!(parse_args(&args(&["run", "a", "extra"])).is_err());
}
