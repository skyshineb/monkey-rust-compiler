#[path = "common/conformance.rs"]
mod conformance;

use conformance::{
    normalize_final_newline, normalize_line_endings, normalize_output, normalize_stacktrace_paths,
    trim_line_trailing_space,
};

#[test]
fn normalizes_crlf_and_final_newline() {
    let raw = "a\r\nb\r\n";
    assert_eq!(normalize_line_endings(raw), "a\nb\n");
    assert_eq!(normalize_final_newline("a\nb\n\n"), "a\nb\n");
}

#[test]
fn trims_line_trailing_whitespace_only() {
    let raw = "a  \n b\t\n";
    assert_eq!(trim_line_trailing_space(raw), "a\n b");
}

#[test]
fn output_normalization_is_deterministic() {
    let raw = "Error[TYPE]  \r\nStack trace:\r\n";
    assert_eq!(normalize_output(raw), "Error[TYPE]\nStack trace:\n");
}

#[test]
fn stacktrace_path_normalization_preserves_frame_text() {
    let raw = "at /tmp/project/target/debug/monkey(0 args) @ 1:1\n";
    let normalized = normalize_stacktrace_paths(raw);
    assert!(normalized.contains("monkey(0 args) @ 1:1"));
}
