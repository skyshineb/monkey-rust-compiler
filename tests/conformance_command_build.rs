#[path = "common/conformance.rs"]
mod conformance;

use conformance::{parse_command_line, unified_diff};

#[test]
fn parses_simple_command_line() {
    let (program, args) = parse_command_line("java -jar ref.jar").expect("parse command");
    assert_eq!(program, "java");
    assert_eq!(args, vec!["-jar", "ref.jar"]);
}

#[test]
fn parses_quoted_command_line() {
    let (program, args) =
        parse_command_line("\"java\" '-jar' \"path with space/ref.jar\"").expect("parse command");
    assert_eq!(program, "java");
    assert_eq!(args, vec!["-jar", "path with space/ref.jar"]);
}

#[test]
fn rejects_empty_command() {
    let err = parse_command_line("   ").expect_err("expected error");
    assert_eq!(err, "empty command");
}

#[test]
fn renders_unified_diff() {
    let diff = unified_diff("left", "a\nb\n", "right", "a\nc\n");
    assert!(diff.contains("--- left"));
    assert!(diff.contains("+++ right"));
    assert!(diff.contains("-b"));
    assert!(diff.contains("+c"));
}
