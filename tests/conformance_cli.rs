#[path = "common/conformance.rs"]
mod conformance;

use std::path::Path;

use conformance::{
    compare_fixture, compare_rust_to_rust, fixture_cases, ConformanceMode, ConformanceOutcome,
};

fn run_mode(mode: ConformanceMode) {
    let fixtures = fixture_cases(mode.fixture_dir());
    for fixture in fixtures {
        match compare_fixture(mode, &fixture) {
            ConformanceOutcome::Match => {}
            ConformanceOutcome::Skipped(reason) => {
                eprintln!("conformance skipped for {}: {reason}", fixture.display());
                return;
            }
            ConformanceOutcome::Mismatch(m) => {
                panic!(
                    "conformance mismatch\nfixture: {}\nmode: {}\nrust: {}\njava: {}\ndiff:\n{}",
                    m.fixture.display(),
                    m.mode,
                    m.rust_cmd,
                    m.java_cmd,
                    m.diff
                );
            }
        }
    }
}

#[test]
fn conformance_skips_cleanly_without_java_config() {
    if std::env::var("MONKEY_JAVA_REF_CMD").is_ok() {
        return;
    }

    let fixture = Path::new("tests/fixtures/conformance/run/arithmetic.monkey");
    match compare_fixture(ConformanceMode::Run, fixture) {
        ConformanceOutcome::Skipped(reason) => {
            assert_eq!(reason, "MONKEY_JAVA_REF_CMD is not set");
        }
        other => panic!("expected skipped outcome, got {other:?}"),
    }
}

#[test]
fn conformance_rust_self_smoke() {
    let fixture = Path::new("tests/fixtures/conformance/run/arithmetic.monkey");
    let result = compare_rust_to_rust(ConformanceMode::Run, fixture).expect("rust self-compare");
    assert_eq!(result, "match");
}

#[test]
fn conformance_run_parity() {
    run_mode(ConformanceMode::Run);
}

#[test]
fn conformance_tokens_parity() {
    run_mode(ConformanceMode::Tokens);
}

#[test]
fn conformance_ast_parity() {
    run_mode(ConformanceMode::Ast);
}
