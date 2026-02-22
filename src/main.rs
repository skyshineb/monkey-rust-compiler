use std::env;
use std::process::ExitCode;

const USAGE: &str = "Usage: monkey [run <path> | bench <path> | --tokens <path> | --ast <path>]";

fn print_usage(stderr: bool) {
    if stderr {
        eprintln!("{USAGE}");
    } else {
        println!("{USAGE}");
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        // TODO(step-3): default no-arg behavior should start REPL for protocol compatibility.
        print_usage(true);
        return ExitCode::from(2);
    }

    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        print_usage(false);
        return ExitCode::SUCCESS;
    }

    print_usage(true);
    ExitCode::from(2)
}
