use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_monkey")
}

#[test]
fn run_mode_smoke() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("examples/hello.monkey");

    let output = Command::new(bin())
        .args(["run", path.to_str().expect("utf8 path")])
        .output()
        .expect("failed to execute monkey binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello from monkey"));
}

#[test]
fn tokens_and_ast_modes_smoke() {
    let mut tokens_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    tokens_path.push("examples/control_flow.monkey");

    let tokens = Command::new(bin())
        .args(["--tokens", tokens_path.to_str().expect("utf8 path")])
        .output()
        .expect("failed to execute monkey --tokens");
    assert!(tokens.status.success());
    assert!(String::from_utf8_lossy(&tokens.stdout).contains("While('while')"));

    let mut ast_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ast_path.push("examples/closures.monkey");

    let ast = Command::new(bin())
        .args(["--ast", ast_path.to_str().expect("utf8 path")])
        .output()
        .expect("failed to execute monkey --ast");
    assert!(ast.status.success());
    assert!(String::from_utf8_lossy(&ast.stdout).contains("fn(a)"));
}
