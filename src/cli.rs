/// Parsed CLI command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Repl,
    Run { path: String },
    Bench { path: String },
    Tokens { path: String },
    Ast { path: String },
    Help,
}

pub fn parse_args(args: &[String]) -> Result<Command, ()> {
    match args {
        [] => Ok(Command::Repl),
        [one] if one == "repl" => Ok(Command::Repl),
        [one] if one == "--help" || one == "-h" => Ok(Command::Help),
        [cmd, path] if cmd == "run" => Ok(Command::Run { path: path.clone() }),
        [cmd, path] if cmd == "bench" => Ok(Command::Bench { path: path.clone() }),
        [cmd, path] if cmd == "--tokens" => Ok(Command::Tokens { path: path.clone() }),
        [cmd, path] if cmd == "--ast" => Ok(Command::Ast { path: path.clone() }),
        _ => Err(()),
    }
}
